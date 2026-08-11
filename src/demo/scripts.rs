use glam::Vec3;

use crate::demo::solvers::PhysicsSolver;
use crate::scripting::id::ScriptId;
use crate::scripting::script::Script;
use crate::scripting::DomainView;
use crate::tables::PipId;

pub struct MyScript {
    pub player: Option<PipId>,
    pub other_script: Option<ScriptId>,
    pub timer: f32,
}

pub struct OtherScript {
    pub num: u32,
}

impl Script for OtherScript {
    fn update(&mut self, _ctx: &mut DomainView) {}
}

const DISABLE_AFTER: f32 = 5.0;

impl Script for MyScript {
    fn update(&mut self, ctx: &mut DomainView) {
        let _ = ctx.with_script_option_mut::<OtherScript>(&mut self.other_script, |other| {
            other.every.enabled = true;
            let _ = other.script.num;
            other.script.num = 42;
        });

        self.timer += ctx.dt();

        if self.timer >= DISABLE_AFTER {
            let _ = ctx.with_solver_mut::<PhysicsSolver>(|physics| {
                physics.every.enabled = false;
            });
        } else {
            let _ = ctx.with_solver_mut::<PhysicsSolver>(|physics| {
                physics.script.gravity = Vec3::new(0.0, -9.8, 0.0);
            });
        }
    }
}
