use crate::assets::AssetRegistry;
use crate::input::Input;
use crate::scripting::error::ScriptGetError;
use crate::scripting::host::ScriptHostMut;
use crate::scripting::id::ScriptId;
use crate::scripting::script::Script;
use crate::scripting::scripts::Scripts;
use crate::scripting::solvers::Solvers;
use crate::tables::domain::Domain;
use crate::tables::tables::Tables;

pub struct DomainView<'a> {
    pub dt: f32,
    pub domain: &'a mut Domain,
    pub scripts: &'a Scripts,
    pub solvers: &'a Solvers,
    pub input: &'a Input,
    pub asset_registry: &'a AssetRegistry,
}

impl<'a> DomainView<'a> {
    pub(crate) fn new(
        dt: f32,
        domain: &'a mut Domain,
        scripts: &'a Scripts,
        solvers: &'a Solvers,
        input: &'a Input,
        asset_registry: &'a AssetRegistry,
    ) -> Self {
        Self {
            dt,
            domain,
            scripts,
            solvers,
            input,
            asset_registry,
        }
    }

    pub fn split(&mut self) -> (&slotmap::SlotMap<crate::tables::PipId, crate::tables::ClassRowPtr>, &mut Tables) {
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
