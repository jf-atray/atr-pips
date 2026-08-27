use crate::addition::Addition;
use crate::addition;
use crate::brushes::Brush;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::ecs::PipId;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;

#[derive(Debug)]
pub struct MotionSolver;

addition! {
    #[derive(Debug)]
    CoreWorld {
        tables: CoreTables {
            xforms: Class<Transform, ()> = Class::new(GrowthStrategy::quart_kib::<Transform>()),
            brushes: Class<Brush> = Class::new(GrowthStrategy::quart_kib::<Brush>()),
            names: Class<String> = Class::new(GrowthStrategy::quart_kib::<String>()),
            motions: Class<Motion> = Class::new(GrowthStrategy::quart_kib::<Motion>()),
            pip_ids: Class<PipId> = Class::new(GrowthStrategy::quart_kib::<PipId>()),
        },
        solvers: CoreSolvers {
            motion: MotionSolver = MotionSolver,
        },
        scripts: CoreScripts {},
        signals: CoreSignals {},
    }
}

impl MotionSolver {
    fn update(
        &mut self,
        dt: f32,
        tables: &mut addition::TypedMap<dyn addition::Tables>,
        _scripts: &mut addition::TypedMap<dyn addition::Scripts>,
        _signals: &mut addition::TypedMap<dyn addition::Signals>,
    ) {
        let Some(core) = CoreWorld::tables(tables) else {
            return;
        };
        crate::query!(
            [&mut core.motions, &mut core.xforms],
            |motion: &mut Motion, xform: &mut Transform| {
                xform.xyz += motion.vel * dt;
            }
        );
    }
}
