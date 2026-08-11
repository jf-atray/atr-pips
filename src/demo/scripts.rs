use crate::demo::solvers::PhysicsSolver;
use crate::scripting::id::ScriptId;
use crate::scripting::script::Script;
use crate::scripting::DomainView;
use crate::tables::PipId;

pub struct MyScript {
    pub player: Option<PipId>,
    pub other_script: Option<ScriptId>,
}

pub struct OtherScript {
    pub num: u32,
}

impl Script for OtherScript {
    fn update(&mut self, _ctx: &DomainView) {}
}

impl Script for MyScript {
    fn update(&mut self, ctx: &DomainView) {
        let _ = ctx.with_script_option_mut::<OtherScript>(&mut self.other_script, |other| {
            other.every.enabled = true;
            let _ = other.script.num;
            other.script.num = 42;
        });

        if self.other_script.is_some() {
            let _ = ctx.with_solver_mut::<PhysicsSolver>(|physics| {
                physics.every.enabled = false;
                physics.script.gravity = 0.0;
            });
        }
    }
}
