use crate::gather::impls::gather_ref;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::ecs::PipId;
use crate::you_first::gamejam::roller::biome::Biome;
use crate::you_first::gamejam::roller::components::RollerAddition;

const BIOME_SWAP_DISTANCE: f32 = 0.1;

#[derive(Debug)]
enum Lifecycle {
    Pending,
    Sparse(Biome),
    Default(Biome),
}

impl Lifecycle {
    const fn biome_mut(&mut self) -> Option<&mut Biome> {
        match self {
            Self::Pending => None,
            Self::Sparse(biome) | Self::Default(biome) => Some(biome),
        }
    }
}

#[derive(Debug)]
pub struct RollerSpawner {
    player: PipId,
    biome: Lifecycle,
}

impl RollerSpawner {
    pub const fn new(player: PipId) -> Self {
        Self {
            player,
            biome: Lifecycle::Pending,
        }
    }

    fn player_walk_distance(&self, ctx: &mut DomainView) -> Option<f32> {
        let roller = ctx.domain.tables.get::<RollerAddition>()?;
        let player = gather_ref(&ctx.domain.ids, &roller.roller_players, self.player)?;
        Some(player.walk_distance)
    }
}

impl Script for RollerSpawner {
    fn update(&mut self, ctx: &mut DomainView) {
        let Some(walk_distance) = self.player_walk_distance(ctx) else {
            return;
        };

        match &mut self.biome {
            Lifecycle::Pending => {
                let mut biome = Biome::sparse_desert();
                biome.pre_seed(ctx.domain, ctx.asset_registry);
                self.biome = Lifecycle::Sparse(biome);
            }
            Lifecycle::Sparse(biome) if walk_distance >= BIOME_SWAP_DISTANCE => {
                let mut next = Biome::default_desert();
                next.pre_seed(ctx.domain, ctx.asset_registry);
                self.biome = Lifecycle::Default(next);
            },
            _ => {}
        }

        if let Some(biome) = self.biome.biome_mut() {
            biome.update(ctx.domain, ctx.asset_registry, walk_distance);
        }
    }
}
