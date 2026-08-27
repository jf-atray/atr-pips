use std::collections::HashMap;

use crate::assets::SpriteEntry;
use crate::gamescope::scene::SceneAction;
use crate::input::Input;
use crate::scripting::error::ScriptGetError;
use crate::scripting::host::ScriptHostMut;
use crate::scripting::id::ScriptId;
use crate::scripting::script::Script;
use crate::scripting::scripts::Scripts;
use crate::scripting::solvers::Solvers;
use crate::addition::ExampleDomain;
use crate::addition::TypedMap;
use crate::addition::Tables as AdditionTables;

pub struct DomainView<'a> {
    pub dt: f32,
    pub domain: &'a mut ExampleDomain,
    pub scripts: &'a Scripts,
    pub solvers: &'a Solvers,
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

    pub fn split(
        &mut self,
    ) -> (
        &slotmap::SlotMap<crate::ecs::PipId, crate::ecs::ClassRowPtr>,
        &mut TypedMap<dyn AdditionTables>,
    ) {
        (&self.domain.ids, &mut self.domain.tables)
    }

    pub fn with_script_mut<T: Script>(
        &self,
        id: ScriptId,
        f: impl FnOnce(ScriptHostMut<'_, T>),
    ) -> Result<(), ScriptGetError> {
        self.scripts.with_mut(id, f)
    }

    pub fn with_script_option_mut<T: Script>(
        &self,
        id: &mut Option<ScriptId>,
        f: impl FnOnce(ScriptHostMut<'_, T>),
    ) -> Result<(), ScriptGetError> {
        self.scripts.with_option_mut(id, f)
    }

    pub fn with_solver_mut<T: Script>(
        &self,
        f: impl FnOnce(ScriptHostMut<'_, T>),
    ) -> Result<(), ScriptGetError> {
        self.solvers.with_mut(f)
    }
}
