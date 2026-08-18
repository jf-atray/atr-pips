use crate::gather::impls::gather_ref;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::tables::PipId;
use crate::you_first::gamejam::roller::biome::Biome;
use crate::you_first::gamejam::roller::components::RollerAddition;

const BIOME_SWAP_DISTANCE: f32 = 0.1;

pub struct RollerSpawner {
    player: PipId,
    biome: Biome,
    initialized: bool,
    biome_swapped: bool,
}

impl RollerSpawner {
    pub fn new(player: PipId) -> Self {
        Self {
            player,
            biome: Biome::sparse_desert(),
            initialized: false,
            biome_swapped: false,
        }
    }
}

impl Script for RollerSpawner {
    fn update(&mut self, ctx: &mut DomainView) {
        let walk_distance = {
            let tables = &ctx.domain.tables;
            let Some(roller) = tables.get::<RollerAddition>() else {
                return;
            };
            let Some(player) = gather_ref(&ctx.domain.ids, &roller.roller_players, self.player)
            else {
                return;
            };

            player.walk_distance
        };

        if !self.initialized {
            self.biome.pre_seed(ctx.domain, ctx.asset_registry);
            self.initialized = true;
        }

        if !self.biome_swapped && walk_distance >= BIOME_SWAP_DISTANCE {
            self.biome = Biome::default_desert();
            self.biome.pre_seed(ctx.domain, ctx.asset_registry);
            self.biome_swapped = true;
        }

        self.biome
            .update(ctx.domain, ctx.asset_registry, walk_distance);
    }
}
