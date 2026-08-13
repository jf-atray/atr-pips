use std::any::Any;

use crate::scripting::error::ScriptGetError;
use crate::scripting::every::EveryScript;
use crate::scripting::script::Script;

pub struct ScriptHostMut<'a, T> {
    pub every: &'a mut EveryScript,
    pub script: &'a mut T,
}

pub struct ScriptHost {
    pub(crate) every: EveryScript,
    pub(crate) script: Box<dyn Script>,
}

impl ScriptHost {
    pub fn new(every: EveryScript, script: Box<dyn Script>) -> Self {
        Self { every, script }
    }

    pub(crate) fn downcast_mut<T: Script>(
        &mut self,
    ) -> Result<ScriptHostMut<'_, T>, ScriptGetError> {
        if (self.script.as_ref() as &dyn Any).is::<T>() {
            let script_as_any = self.script.as_mut() as &mut dyn Any;
            let Some(script_as_cast) = script_as_any.downcast_mut::<T>() else {
                unreachable!()
            };
            Ok(ScriptHostMut {
                every: &mut self.every,
                script: script_as_cast,
            })
        } else {
            Err(ScriptGetError::BadCast)
        }
    }
}
