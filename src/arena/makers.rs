use crate::arena::tables::{
    ActorView, ArenaView, HealthData, HealthPickupData, PilotData, ProjectileData, Team,
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
        if let (Some(team), Some(pilot)) = (self.team, self.pilot) {
            scope.view::<ActorView>().map(|view| view.pilot(team, pilot));
        } else {
            self.team.map(|team| scope.view::<ActorView>().map(|view| view.team(team)));
        }
        self.health.map(|health| scope.view::<ArenaView>().map(|view| view.health(health)));
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
        scope.view::<ArenaView>().map(|view| view.projectile(self.projectile));
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
        self.team.map(|team| scope.view::<ActorView>().map(|view| view.team(team)));
        scope.view::<ArenaView>().map(|view| view.pickup(self.pickup));
    }
}
