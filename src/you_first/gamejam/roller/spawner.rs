use crate::gather::impls::gather_mut;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::tables::PipId;
use crate::you_first::gamejam::roller::biome::Biome;
use crate::you_first::gamejam::roller::components::{RollerAddition, RollerPlayer};
use crate::you_first::gamejam::roller::projection::WALK_SPEED;

pub struct RollerSpawner {
    player: PipId,
    biome: Biome,
    rng: u32,
}

impl RollerSpawner {
    pub fn new(player: PipId, biome: Biome, rng: u32) -> Self {
        Self { player, biome, rng }
    }
}

impl Script for RollerSpawner {
    fn update(&mut self, ctx: &mut DomainView) {
        let walk_distance = {
            let tables = &mut ctx.domain.tables;
            let Some(roller) = tables.get_mut::<RollerAddition>() else {
                return;
            };
            let Some(player) = gather_mut(&ctx.domain.ids, &mut roller.roller_players, self.player) else {
                return;
            };

            player.walk_distance += WALK_SPEED * ctx.dt;
            player.walk_distance
        };

        self.biome
            .update(&mut ctx.domain, ctx.asset_registry, walk_distance, &mut self.rng);
    }
}
