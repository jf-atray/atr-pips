mod hash;
mod solver;

pub use hash::{SpatialHash, CandidatePairs};
pub use solver::BroadPhaseSolver;

use crate::addition;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::spacial::aabb::Aabb;

addition! {
    #[derive(Debug)]
    pub struct broadphase_world : BroadPhaseAdd {
        tables: {
            aabbs: Class<Aabb> = Class::new(GrowthStrategy::quart_kib::<Aabb>()),
        },
        solvers: { broadphase: BroadPhaseSolver = BroadPhaseSolver::new() },
        scripts: {},
        signals: {
            hash: SpatialHash = SpatialHash::new(2.0),
            pairs: CandidatePairs = CandidatePairs::default(),
        },
    }
}
