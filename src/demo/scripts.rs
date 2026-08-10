use crate::scripting::id::ScriptId;
use crate::scripting::script::Script;
use crate::scripting::scripts::Scripts;
use crate::tables::PipId;

pub struct MyScript {
    pub player: Option<PipId>,
    pub other_script: Option<ScriptId>,
}

pub struct OtherScript {
    pub num: u32,
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
