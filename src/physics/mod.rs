pub mod data;
pub mod solver;
mod gravity;

use crate::addition;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::physics::data::impulse::Impulse;
use crate::physics::data::mass::InvMass;
use crate::physics::gravity::Gravity;
use crate::physics::solver::PhysicsSolver;

addition! {
    #[derive(Debug)]
    pub struct physics_world : PhysicsAdd {
        tables: {
            inv_masses: Class<InvMass> = Class::new(GrowthStrategy::quart_kib::<InvMass>()),
            impulses: Class<Impulse> = Class::new(GrowthStrategy::quart_kib::<Impulse>()),
        },
        solvers: { physics_solver: PhysicsSolver = PhysicsSolver },
        scripts: {},
        signals: { gravity: Gravity = Gravity::default() },
    }
}
