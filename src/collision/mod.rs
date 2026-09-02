mod broad_solver;
mod contact;
mod hash;
mod narrow_solver;
mod obb;

pub use broad_solver::BroadPhaseSolver;
#[allow(unused_imports)]
pub use contact::{ContactCache, ContactPair, ManifoldPoint};
pub use hash::{CandidatePairs, SpatialHash};
pub use narrow_solver::NarrowPhaseSolver;

use crate::addition;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::spacial::aabb::Aabb;

addition! {
    #[derive(Debug)]
    pub struct collision_world : CollisionAdd {
        tables: {
            aabbs: Class<Aabb> = Class::new(GrowthStrategy::quart_kib::<Aabb>()),
        },
        solvers: {
            broad: BroadPhaseSolver = BroadPhaseSolver::new(),
            narrow: NarrowPhaseSolver = NarrowPhaseSolver::new(),
        },
        scripts: {},
        signals: {
            hash: SpatialHash = SpatialHash::new(5.0),
            pairs: CandidatePairs = CandidatePairs::default(),
        },
    }
}
