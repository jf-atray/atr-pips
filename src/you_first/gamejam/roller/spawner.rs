use crate::gather::impls::gather_ref;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::tables::PipId;
use crate::you_first::gamejam::roller::biome::Biome;
use crate::you_first::gamejam::roller::components::RollerAddition;

pub struct RollerSpawner {
    player: PipId,
    biome: Biome,
}

impl RollerSpawner {
    pub fn new(player: PipId, biome: Biome) -> Self {
        Self { player, biome }
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

        self.biome
            .update(ctx.domain, ctx.asset_registry, walk_distance);
    }
}
