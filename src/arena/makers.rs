use crate::arena::tables::{
    HealthData, HealthPickupData, HealthPickupView, HealthView, PilotData, PilotView,
    ProjectileData, ProjectileView, SpawnerData, SpawnerView, Team, TeamView,
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
        if let Some(team) = self.team {
            if let Some(view) = scope.view::<TeamView>() {
                view.team = Some(team);
            }
        }
        if let Some(health) = self.health {
            if let Some(view) = scope.view::<HealthView>() {
                view.data = Some(health);
            }
        }
        if let Some(pilot) = self.pilot {
            if let Some(view) = scope.view::<PilotView>() {
                view.data = Some(pilot);
            }
        }
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
        if let Some(view) = scope.view::<ProjectileView>() {
            view.data = Some(self.projectile);
        }
    }
}

pub struct SpawnerBlueprint {
    pub xform: Transform,
    pub brush: Brush,
    pub name: Option<String>,
    pub team: Option<Team>,
    pub spawner: SpawnerData,
}

impl Maker for SpawnerBlueprint {
    fn make_into(self, scope: &mut Scope) {
        scope.core.xforms = Some(self.xform);
        scope.core.brushes = Some(self.brush);
        if let Some(name) = self.name {
            scope.core.names = Some(name);
        }
        if let Some(team) = self.team {
            if let Some(view) = scope.view::<TeamView>() {
                view.team = Some(team);
            }
        }
        if let Some(view) = scope.view::<SpawnerView>() {
            view.data = Some(self.spawner);
        }
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
        if let Some(team) = self.team {
            if let Some(view) = scope.view::<TeamView>() {
                view.team = Some(team);
            }
        }
        if let Some(view) = scope.view::<HealthPickupView>() {
            view.data = Some(self.pickup);
        }
    }
}

