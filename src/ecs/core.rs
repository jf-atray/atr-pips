use crate::addition;
use crate::brushes::Brush;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::spacial::boundary::Boundary;
use crate::spacial::motion::{Motion, MotionKind};
use crate::spacial::transform::Transform;

addition! {
    #[derive(Debug)]
    pub struct core_tables : CoreAdd {
        tables: {
            xforms: Class<Transform> = Class::new(GrowthStrategy::quart_kib::<Transform>()),
            brushes: Class<Brush> = Class::new(GrowthStrategy::quart_kib::<Brush>()),
            motions: Class<Motion, MotionKind> = Class::new(GrowthStrategy::quart_kib::<Motion>()),
        },
        solvers: { motion: crate::gamescope::motion::MotionSolver = crate::gamescope::motion::MotionSolver::new() },
        scripts: {},
        signals: {
            boundary: Boundary = Boundary::default(),
            drag: f32 = 0.0,
        },
    }
}