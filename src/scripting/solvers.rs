use std::any::TypeId;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

use crate::scripting::context::DomainView;
use crate::scripting::error::ScriptGetError;
use crate::scripting::every::EveryScript;
use crate::scripting::host::{ScriptHost, ScriptHostMut};
use crate::scripting::script::Script;
use crate::scripting::scripts::Scripts;
use crate::tables::domain::Domain;

pub struct Solvers {
    solvers: HashMap<TypeId, RefCell<ScriptHost>>,
}

impl Solvers {
    pub fn new() -> Self {
        Self {
            solvers: HashMap::new(),
        }
    }

    pub fn register<T: Script>(&mut self, solver: T) {
        let type_id = TypeId::of::<T>();
        if self.solvers.contains_key(&type_id) {
            panic!(
                "duplicate solver registered: {}",
                std::any::type_name::<T>()
            );
        }
        self.solvers.insert(
            type_id,
            RefCell::new(ScriptHost::new(
                EveryScript { enabled: true },
                Box::new(solver),
            )),
        );
    }

    fn try_borrow_host<'a>(
        cell: &'a RefCell<ScriptHost>,
    ) -> Result<RefMut<'a, ScriptHost>, ScriptGetError> {
        cell.try_borrow_mut().map_err(|_| ScriptGetError::BadAlias)
    }

    fn with_host<T: Script, R>(
        &self,
        f: impl FnOnce(ScriptHostMut<'_, T>) -> R,
    ) -> Result<R, ScriptGetError> {
        let type_id = TypeId::of::<T>();
        let cell = self.solvers.get(&type_id).ok_or(ScriptGetError::BadId)?;
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

    pub fn update_enabled(&self, dt: f32, domain: &mut Domain, scripts: &Scripts) {
        let mut ctx = DomainView::new(dt, domain, scripts, self);
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
