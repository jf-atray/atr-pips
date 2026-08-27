use std::collections::HashMap;

use crate::assets::SpriteEntry;
use crate::gamescope::scene::SceneAction;
use crate::input::Input;
use crate::addition::{ExampleDomain, TablesMap};

pub struct DomainView<'a> {
    pub dt: f32,
    pub domain: &'a mut ExampleDomain,
    pub input: &'a Input,
    pub asset_registry: &'a HashMap<String, SpriteEntry>,
    pub game_action: &'a SceneAction,
}

impl<'a> DomainView<'a> {
    pub(crate) fn new(
        dt: f32,
        domain: &'a mut ExampleDomain,
        scripts: &'a Scripts,
        solvers: &'a Solvers,
        input: &'a Input,
        asset_registry: &'a HashMap<String, SpriteEntry>,
        game_action: &'a SceneAction,
    ) -> Self {
        Self {
            dt,
            domain,
            scripts,
            solvers,
            input,
            asset_registry,
            game_action,
        }
    }
}
