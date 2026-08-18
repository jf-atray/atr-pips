use std::cell::{RefCell, RefMut};

use slotmap::SlotMap;

use crate::assets::AssetRegistry;
use crate::input::Input;
use crate::scripting::context::DomainView;
use crate::scripting::error::ScriptGetError;
use crate::scripting::host::{ScriptHost, ScriptHostMut};
use crate::scripting::id::ScriptId;
use crate::scripting::script::Script;
use crate::scripting::solvers::Solvers;
use crate::tables::domain::Domain;

// making a disjoint thin view to do a dynamic check is just the same as
// doing that check here anyway using std features
pub struct Scripts {
    scripts: SlotMap<ScriptId, RefCell<ScriptHost>>,
}

impl Scripts {
    pub fn new() -> Self {
        Self {
            scripts: SlotMap::with_key(),
        }
    }

    pub fn add(&mut self, host: ScriptHost) -> ScriptId {
        self.scripts.insert(RefCell::new(host))
    }

    fn try_borrow_host(
        cell: &RefCell<ScriptHost>,
    ) -> Result<RefMut<'_, ScriptHost>, ScriptGetError> {
        cell.try_borrow_mut().map_err(|_| ScriptGetError::BadAlias)
    }

    fn with_host<T: Script, R>(
        &self,
        id: ScriptId,
        f: impl FnOnce(ScriptHostMut<'_, T>) -> R,
    ) -> Result<R, ScriptGetError> {
        let cell = self.scripts.get(id).ok_or(ScriptGetError::BadId)?;
        let mut guard = Self::try_borrow_host(cell)?;
        let host = guard.downcast_mut::<T>()?;
        Ok(f(host))
    }

    pub fn with_mut<T: Script>(
        &self,
        id: ScriptId,
        f: impl FnOnce(ScriptHostMut<'_, T>),
    ) -> Result<(), ScriptGetError> {
        self.with_host(id, f)
    }

    pub fn with_mut_and<T: Script>(
        &self,
        id: ScriptId,
        f: impl FnOnce(ScriptHostMut<'_, T>, &Scripts),
    ) -> Result<(), ScriptGetError> {
        self.with_host(id, |host| f(host, self))
    }

    pub fn with_option_mut<T: Script>(
        &self,
        id: &mut Option<ScriptId>,
        f: impl FnOnce(ScriptHostMut<'_, T>),
    ) -> Result<(), ScriptGetError> {
        let Some(f_id) = id else {
            return Err(ScriptGetError::BadId);
        };
        let result = self.with_mut::<T>(*f_id, f);
        if result.is_err() {
            *id = None;
        }
        result
    }

    pub fn set_enabled(&self, id: ScriptId, enabled: bool) -> Result<(), ScriptGetError> {
        let cell = self.scripts.get(id).ok_or(ScriptGetError::BadId)?;
        let mut guard = Self::try_borrow_host(cell)?;
        guard.every.enabled = enabled;
        Ok(())
    }

    pub fn enable(&self, id: ScriptId) -> Result<(), ScriptGetError> {
        self.set_enabled(id, true)
    }

    pub fn disable(&self, id: ScriptId) -> Result<(), ScriptGetError> {
        self.set_enabled(id, false)
    }

    pub fn update_enabled(
        &self,
        dt: f32,
        domain: &mut Domain,
        solvers: &Solvers,
        asset_registry: &AssetRegistry,
        input: &Input,
    ) {
        let mut ctx = DomainView::new(dt, domain, self, solvers, input, asset_registry);
        self.foreach_untyped(|_scripts, host| {
            if host.every.enabled {
                host.script.update(&mut ctx);
            }
        });
    }

    pub fn foreach_untyped(&self, mut f: impl FnMut(&Scripts, &mut ScriptHost)) {
        for cell in self.scripts.values() {
            if let Ok(mut guard) = Self::try_borrow_host(cell) {
                let host = &mut *guard;
                f(self, host);
            }
        }
    }

    pub fn foreach<T: Script>(&self, mut f: impl FnMut(ScriptHostMut<'_, T>)) {
        self.foreach_untyped(|_scripts, host| {
            if let Ok(host) = host.downcast_mut::<T>() {
                f(host);
            }
        });
    }
}
