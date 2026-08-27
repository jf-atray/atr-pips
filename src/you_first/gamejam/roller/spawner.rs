use crate::addition::Addition;
use crate::anims::AnimRules;
use crate::gather::impls::gather_ref;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::ecs::PipId;
use crate::you_first::gamejam::duel::bundle::build_tumbleweed_anim_library;
use crate::you_first::gamejam::duel::state::TumbleweedAnimLib;
use crate::you_first::gamejam::roller::biome::Biome;
use crate::you_first::gamejam::roller::components::RollerWorld;

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
    tumbleweed: Option<(TumbleweedAnimLib, AnimRules)>,
}

impl RollerSpawner {
    pub const fn new(player: PipId) -> Self {
        Self {
            player,
            biome: Lifecycle::Pending,
            tumbleweed: None,
        }
    }

    fn player_walk_distance(&self, ctx: &mut DomainView) -> Option<f32> {
        let roller = RollerWorld::tables(&mut ctx.domain.pips.tables)?;
        let player = gather_ref(&ctx.domain.pips.ids, &roller.roller_players, self.player)?;
        Some(player.walk_distance)
    }
}

impl Script for RollerSpawner {
    fn update(&mut self, ctx: &mut DomainView) {
        let Some(walk_distance) = self.player_walk_distance(ctx) else {
            return;
        };

        if self.tumbleweed.is_none() {
            let (lib, root) = build_tumbleweed_anim_library();
            let lib_id = ctx.domain.pips.anim_libs.insert(lib);
            let rules = ctx
                .domain
                .pips
                .anim_libs
                .get(lib_id)
                .unwrap()
                .get(root)
                .unwrap()
                .rules
                .clone();
            self.tumbleweed = Some((TumbleweedAnimLib { lib: lib_id, root_anim: root }, rules));
        }

        let tumbleweed = self.tumbleweed.clone();

        match &mut self.biome {
            Lifecycle::Pending => {
                let mut biome = Biome::sparse_desert(tumbleweed);
                biome.pre_seed(ctx.domain, ctx.asset_registry);
                self.biome = Lifecycle::Sparse(biome);
            }
            Lifecycle::Sparse(biome) if walk_distance >= BIOME_SWAP_DISTANCE => {
                let mut next = Biome::default_desert(tumbleweed);
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
