use crate::addition;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::ecs::PipId;

addition! {
    #[derive(Debug)]
    pub struct system_world : SystemWorld {
        tables: {
            pip_id: Class<PipId> = Class::new(GrowthStrategy::quart_kib::<PipId>()),
        },
        solvers: {},
        scripts: {},
        signals: {},
    }
}
