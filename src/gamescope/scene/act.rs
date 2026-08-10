use std::{
    any::Any,
    cell::RefCell,
    ops::{Deref, DerefMut},
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
    num: u32,
}

pub struct TopLevel {
    pub scripts: Scripts,
}

impl TopLevel {
    pub fn a(&mut self) -> () {
        let id_2 = self.scripts.scripts.insert(RefCell::new(ScriptHost(
            EveryScript { enabled: false },
            Box::new(OtherScript { num: 0 }),
        )));
        let id_1 = self.scripts.scripts.insert(RefCell::new(ScriptHost(
            EveryScript { enabled: false },
            Box::new(MyScript {
                player: None,
                other_script: Some(id_2),
            }),
        )));
        self.scripts
            .with_mut_and::<MyScript>(id_1, |host, scripts| {
                host.script.update(scripts);
            })
            .unwrap();
    }
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
fn borrow<T: Script, F>(scripts: &Scripts, id: &mut Option<ScriptId>, f: F) -> Result<(), ()>
where
    F: FnOnce(ScriptHostMut<'_, T>),
{
    let Some(f_id) = id else {
        return Err(());
    };
    let ok = scripts.with_mut::<T>(*f_id, f);
    if ok.is_err() {
        *id = None;
        return Err(());
    }
    return Ok(());
}

pub trait Script: Any {
    fn update(&mut self, scripts: &Scripts);
}

impl Script for OtherScript {
    fn update(&mut self, _scripts: &Scripts) {}
}

impl Script for MyScript {
    fn update(&mut self, scripts: &Scripts) {
        let mut found_num = 0;
        borrow::<OtherScript, _>(scripts, &mut self.other_script, |other| {
            other.every.enabled = true;
            found_num = other.script.num;
            other.script.num = 42;
        });
    }
}

impl Scripts {
    pub fn with_mut<T: Script>(
        &self,
        id: ScriptId,
        f: impl FnOnce(ScriptHostMut<'_, T>),
    ) -> Result<(), ScriptGetError> {
        let Some(cell) = self.scripts.get(id) else {
            return Err(ScriptGetError::BadId);
        };

        let mut guard = cell.borrow_mut();
        let ScriptHost(every, one) = &mut *guard;

        if (one.as_ref() as &dyn Any).is::<T>() {
            let one_as_any = one.as_mut() as &mut dyn Any;
            let Some(one_as_cast) = one_as_any.downcast_mut::<T>() else {
                unreachable!()
            };
            f(ScriptHostMut {
                every,
                script: one_as_cast,
            });
            Ok(())
        } else {
            Err(ScriptGetError::BadCast)
        }
    }

    pub fn with_mut_and<T: Script>(
        &self,
        id: ScriptId,
        f: impl FnOnce(ScriptHostMut<'_, T>, &Scripts),
    ) -> Result<(), ScriptGetError> {
        let Some(cell) = self.scripts.get(id) else {
            return Err(ScriptGetError::BadId);
        };

        let mut guard = cell.borrow_mut();
        let ScriptHost(every, one) = &mut *guard;

        if (one.as_ref() as &dyn Any).is::<T>() {
            let one_as_any = one.as_mut() as &mut dyn Any;
            let Some(one_as_cast) = one_as_any.downcast_mut::<T>() else {
                unreachable!()
            };
            f(
                ScriptHostMut {
                    every,
                    script: one_as_cast,
                },
                self,
            );
            Ok(())
        } else {
            Err(ScriptGetError::BadCast)
        }
    }
}
