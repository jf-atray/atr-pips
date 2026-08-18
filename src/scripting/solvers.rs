use std::any::TypeId;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

use slotmap::SlotMap;

use crate::assets::AssetRegistry;
use crate::input::Input;
use crate::scripting::context::DomainView;
use crate::scripting::error::ScriptGetError;
use crate::scripting::every::EveryScript;
use crate::scripting::host::{ScriptHost, ScriptHostMut};
use crate::scripting::id::SolverId;
use crate::scripting::script::Script;
use crate::scripting::scripts::Scripts;
use crate::tables::domain::Domain;

pub struct Solvers {
    solvers: SlotMap<SolverId, RefCell<ScriptHost>>,
    by_type: HashMap<TypeId, SolverId>,
}

impl Solvers {
    pub fn new() -> Self {
        Self {
            solvers: SlotMap::with_key(),
            by_type: HashMap::new(),
        }
    }

    pub fn register<T: Script>(&mut self, solver: T) {
        let type_id = TypeId::of::<T>();
        assert!(
            !self.by_type.contains_key(&type_id),
            "duplicate solver registered: {}",
            std::any::type_name::<T>()
        );
        let id = self.solvers.insert(RefCell::new(ScriptHost::new(
            EveryScript { enabled: true },
            Box::new(solver),
        )));
        self.by_type.insert(type_id, id);
    }

    pub fn remove<T: Script>(&mut self) -> Option<ScriptHost> {
        let type_id = TypeId::of::<T>();
        let id = self.by_type.remove(&type_id)?;
        self.solvers.remove(id).map(std::cell::RefCell::into_inner)
    }

    fn try_borrow_host(
        cell: &RefCell<ScriptHost>,
    ) -> Result<RefMut<'_, ScriptHost>, ScriptGetError> {
        cell.try_borrow_mut().map_err(|_| ScriptGetError::BadAlias)
    }

    fn with_host<T: Script, R>(
        &self,
        f: impl FnOnce(ScriptHostMut<'_, T>) -> R,
    ) -> Result<R, ScriptGetError> {
        let type_id = TypeId::of::<T>();
        let id = self
            .by_type
            .get(&type_id)
            .copied()
            .ok_or(ScriptGetError::BadId)?;
        let cell = &self.solvers[id];
        let mut guard = Self::try_borrow_host(cell)?;
        let host = guard.downcast_mut::<T>()?;
        Ok(f(host))
    }

    pub fn with_mut<T: Script>(
        &self,
        f: impl FnOnce(ScriptHostMut<'_, T>),
    ) -> Result<(), ScriptGetError> {
        self.with_host(f)
    }

    pub fn set_enabled<T: Script>(&self, enabled: bool) -> Result<(), ScriptGetError> {
        self.with_mut::<T>(|host| host.every.enabled = enabled)
    }

    pub fn enable<T: Script>(&self) -> Result<(), ScriptGetError> {
        self.set_enabled::<T>(true)
    }

    pub fn disable<T: Script>(&self) -> Result<(), ScriptGetError> {
        self.set_enabled::<T>(false)
    }

    pub fn update_enabled(
        &self,
        dt: f32,
        domain: &mut Domain,
        scripts: &Scripts,
        asset_registry: &AssetRegistry,
        input: &Input,
    ) {
        let mut ctx = DomainView::new(dt, domain, scripts, self, input, asset_registry);
        for cell in self.solvers.values() {
            if let Ok(mut guard) = Self::try_borrow_host(cell) {
                let host = &mut *guard;
                if host.every.enabled {
                    host.script.update(&mut ctx);
                }
            }
        }
    }
}
