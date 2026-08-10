use std::{
    any::Any,
    cell::{RefCell, RefMut},
};

use slotmap::{SlotMap, new_key_type};

use crate::tables::PipId;

//do not communicate by sharing memory, share memory by communicating
//so ive already completely dropped the ball on that but let me take this
//to its logical conclusion so I can learn to be better. :)
pub struct Act {}

pub struct EveryScript {
    pub enabled: bool,
}

pub struct ScriptHost(EveryScript, Box<dyn Script>);

impl ScriptHost {
    pub fn new(every: EveryScript, one: Box<dyn Script>) -> Self {
        Self(every, one)
    }
}

pub struct ScriptHostMut<'a, T> {
    pub every: &'a mut EveryScript,
    pub script: &'a mut T,
}

new_key_type! {
    pub struct ScriptId;
}

pub struct MyScript {
    pub player: Option<PipId>,
    pub other_script: Option<ScriptId>,
}

pub struct OtherScript {
    pub num: u32,
}

// making a disjoint thin view to do a dynamic check is just the same as
// doing that check here anyway using std features
pub struct Scripts {
    pub scripts: SlotMap<ScriptId, RefCell<ScriptHost>>,
}

#[derive(Debug)]
pub enum ScriptGetError {
    BadId,
    BadCast,
    BadAlias,
}

pub trait Script: Any {
    fn update(&mut self, scripts: &Scripts);
}

impl Script for OtherScript {
    fn update(&mut self, _scripts: &Scripts) {}
}

impl Script for MyScript {
    fn update(&mut self, scripts: &Scripts) {
        let _ = scripts.with_option_mut::<OtherScript>(&mut self.other_script, |other| {
            other.every.enabled = true;
            let _ = other.script.num;
            other.script.num = 42;
        });
    }
}

impl Scripts {
    pub fn new() -> Self {
        Self {
            scripts: SlotMap::with_key(),
        }
    }

    fn try_borrow_host<'a>(
        cell: &'a RefCell<ScriptHost>,
    ) -> Result<RefMut<'a, ScriptHost>, ScriptGetError> {
        cell.try_borrow_mut().map_err(|_| ScriptGetError::BadAlias)
    }

    fn downcast_host<'a, T: Script>(
        host: &'a mut ScriptHost,
    ) -> Result<ScriptHostMut<'a, T>, ScriptGetError> {
        let ScriptHost(every, one) = host;

        if (one.as_ref() as &dyn Any).is::<T>() {
            let one_as_any = one.as_mut() as &mut dyn Any;
            let Some(one_as_cast) = one_as_any.downcast_mut::<T>() else {
                unreachable!()
            };
            Ok(ScriptHostMut {
                every,
                script: one_as_cast,
            })
        } else {
            Err(ScriptGetError::BadCast)
        }
    }

    fn with_host<T: Script, R>(
        &self,
        id: ScriptId,
        f: impl FnOnce(ScriptHostMut<'_, T>) -> R,
    ) -> Result<R, ScriptGetError> {
        let cell = self.scripts.get(id).ok_or(ScriptGetError::BadId)?;
        let mut guard = Self::try_borrow_host(cell)?;
        let host = Self::downcast_host::<T>(&mut *guard)?;
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
        guard.0.enabled = enabled;
        Ok(())
    }

    pub fn enable(&self, id: ScriptId) -> Result<(), ScriptGetError> {
        self.set_enabled(id, true)
    }

    pub fn disable(&self, id: ScriptId) -> Result<(), ScriptGetError> {
        self.set_enabled(id, false)
    }

    pub fn update_enabled(&self) {
        self.foreach_untyped(|scripts, host| {
            if host.0.enabled {
                host.1.update(scripts);
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
            if let Ok(host) = Self::downcast_host::<T>(host) {
                f(host);
            }
        });
    }
}
