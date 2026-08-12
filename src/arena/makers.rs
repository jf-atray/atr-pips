use crate::arena::tables::{
    HealthData, HealthPickupData, HealthPickupView, HealthView, PilotData, PilotView,
    ProjectileData, ProjectileView, Team, TeamView,
};
use crate::brushes::Brush;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::scope::{Maker, Scope};

pub struct ActorBlueprint {
    pub xform: Transform,
    pub brush: Brush,
    pub name: Option<String>,
    pub motion: Option<Motion>,
    pub team: Option<Team>,
    pub health: Option<HealthData>,
    pub pilot: Option<PilotData>,
}

impl Maker for ActorBlueprint {
    fn make_into(self, scope: &mut Scope) {
        scope.core.xforms = Some(self.xform);
        scope.core.brushes = Some(self.brush);
        if let Some(name) = self.name {
            scope.core.names = Some(name);
        }
        if let Some(motion) = self.motion {
            scope.core.motions = Some(motion);
        }
        self.team.map(|team| scope.view::<TeamView>().map(|view| view.with(team)));
        self.health.map(|health| scope.view::<HealthView>().map(|view| view.with(health)));
        self.pilot.map(|pilot| scope.view::<PilotView>().map(|view| view.with(pilot)));
    }
}

pub struct ProjectileBlueprint {
    pub xform: Transform,
    pub brush: Brush,
    pub motion: Motion,
    pub projectile: ProjectileData,
}

impl Maker for ProjectileBlueprint {
    fn make_into(self, scope: &mut Scope) {
        scope.core.xforms = Some(self.xform);
        scope.core.brushes = Some(self.brush);
        scope.core.motions = Some(self.motion);
        scope.view::<ProjectileView>().map(|view| view.with(self.projectile));
    }
}

pub struct PickupBlueprint {
    pub xform: Transform,
    pub brush: Brush,
    pub name: Option<String>,
    pub team: Option<Team>,
    pub pickup: HealthPickupData,
}

impl Maker for PickupBlueprint {
    fn make_into(self, scope: &mut Scope) {
        scope.core.xforms = Some(self.xform);
        scope.core.brushes = Some(self.brush);
        if let Some(name) = self.name {
            scope.core.names = Some(name);
        }
        self.team.map(|team| scope.view::<TeamView>().map(|view| view.with(team)));
        scope.view::<HealthPickupView>().map(|view| view.with(self.pickup));
    }
}
