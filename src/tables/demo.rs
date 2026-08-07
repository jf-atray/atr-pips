use atr_plex::duplex;

use crate::tables::{CanvasSolverId, class_strategy};
use crate::tables::class::Class;
use crate::tables::class_strategy::{ClassRarity, GrowthStrategy, rarity};
use crate::brushes::Brush;
use crate::spacial::transform::Transform;

pub struct DemoTables {
    pub xforms: Class<Transform, ()>,
    pub brushes: Class<Brush, CanvasSolverId>,
}

impl DemoTables {
    pub fn new() -> Self {
        Self {
            xforms: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Transform>()),
            brushes: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Brush>()),
        }
    }
}
