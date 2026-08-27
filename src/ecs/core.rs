use crate::addition;
use crate::brushes::Brush;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;

addition! {
    #[derive(Debug)]
    pub struct core_tables : CoreTablesWorld {
        tables: {
            xforms: Class<Transform, ()> = Class::new(GrowthStrategy::quart_kib::<Transform>()),
            brushes: Class<Brush> = Class::new(GrowthStrategy::quart_kib::<Brush>()),
            names: Class<String> = Class::new(GrowthStrategy::quart_kib::<String>()),
            motions: Class<Motion> = Class::new(GrowthStrategy::quart_kib::<Motion>()),
        },
        solvers: {},
        scripts: {},
        signals: {},
    }
}