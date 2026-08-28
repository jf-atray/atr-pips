use std::collections::HashMap;

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::anims::AnimRules;
use crate::assets::SpriteEntry;
use crate::ecs::PipId;
use crate::gather::impls::gather_ref;
use crate::input::Input;
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
    pub player: Option<PipId>,
    biome: Lifecycle,
    tumbleweed: Option<(TumbleweedAnimLib, AnimRules)>,
}

impl Solver for RollerSpawner {}

impl RollerSpawner {
    pub const fn new(player: Option<PipId>) -> Self {
        Self {
            player,
            biome: Lifecycle::Pending,
            tumbleweed: None,
        }
    }

    fn player_walk_distance(&self, pips: &mut Pips) -> Option<f32> {
        let player = self.player?;
        let roller = RollerWorld::tables(&mut pips.tables.pile)?;
        let player = gather_ref(&pips.ids, &roller.roller_players, player)?;
        Some(player.walk_distance)
    }

    pub fn update(
        &mut self,
        _dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        _signals: &mut SignalsMap,
        _input: &Input,
        asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let Some(walk_distance) = self.player_walk_distance(pips) else {
            return;
        };

        if self.tumbleweed.is_none() {
            let (lib, root) = build_tumbleweed_anim_library();
            let lib_id = pips.anim_libs.insert(lib);
            let rules = pips
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
                biome.pre_seed(pips, asset_registry);
                self.biome = Lifecycle::Sparse(biome);
            }
            Lifecycle::Sparse(biome) if walk_distance >= BIOME_SWAP_DISTANCE => {
                let mut next = Biome::default_desert(tumbleweed);
                next.pre_seed(pips, asset_registry);
                self.biome = Lifecycle::Default(next);
            },
            _ => {}
        }

        if let Some(biome) = self.biome.biome_mut() {
            biome.update(pips, asset_registry, walk_distance);
        }
    }
}
