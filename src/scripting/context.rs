use crate::scripting::error::ScriptGetError;
use crate::scripting::host::ScriptHostMut;
use crate::scripting::id::ScriptId;
use crate::scripting::script::Script;
use crate::scripting::scripts::Scripts;
use crate::scripting::solvers::Solvers;
use crate::tables::domain::Domain;

pub struct DomainView<'a> {
    dt: f32,//todo, fixed_step
    domain: &'a mut Domain,
    scripts: &'a Scripts,
    solvers: &'a Solvers,
}

//technkcially allows up to borrow both things at once. hmm
impl<'a> DomainView<'a> {
    pub(crate) fn new(
        dt: f32,
        domain: &'a mut Domain,
        scripts: &'a Scripts,
        solvers: &'a Solvers,
    ) -> Self {
        Self {
            dt,
            domain,
            scripts,
            solvers,
        }
    }

    //this is I suppose the responsible way to readonly
    //even though my dour heart wants to instead destructure the arrangement
    pub fn dt(&self) -> f32 {
        self.dt
    }

    pub fn domain(&mut self) -> &mut Domain {
        self.domain
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
