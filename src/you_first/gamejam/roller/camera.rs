use crate::camera::camera_man::CameraMan;
use crate::gamejam::roller::projection::GROUND_Y;
use crate::pip::Transform;
use crate::scenes::SceneAction;
use crate::scripts::{InputContext, Script, SimulationContext};
use crate::tables::PipId;
use crate::world::World;

const DESIGN_W: f32 = 1280.0;
const DESIGN_H: f32 = 720.0;

const CAMERA_MARGIN: f32 = 3.0;

pub struct OverworldCamera {
    player: PipId,
    camera_x: f32,
}

impl OverworldCamera {
    pub fn new(player: PipId) -> Self {
        Self {
            player,
            camera_x: 0.0,
        }
    }

    fn compute_camera_y(bounds: f32) -> f32 {
        let aspect = DESIGN_H / DESIGN_W / 2.0;
        let horizon_ndc_y = 1.0 / 3.0;
        -horizon_ndc_y * bounds * aspect
    }
}

impl Script for OverworldCamera {
    fn fixed_update(
        &mut self,
        world: &mut World,
        _input: &InputContext,
        ctx: &mut SimulationContext,
    ) -> Option<SceneAction> {
        use crate::gather;
        if let Some(transform) = gather!(self.player, &world.heading, [&world.tables.transforms]) {
            let player_x = transform.xyz.x;
            let delta = player_x - self.camera_x;
            if delta > CAMERA_MARGIN {
                self.camera_x = player_x - CAMERA_MARGIN;
            } else if delta < -CAMERA_MARGIN {
                self.camera_x = player_x + CAMERA_MARGIN;
            }
        }

        let camera_y = Self::compute_camera_y(ctx.camera.camera.bounds);
        *ctx.camera.camera_man =
            CameraMan::move_to(glam::vec3(self.camera_x, camera_y, 0.0));
        None
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizon_maps_to_top_third() {
        const CAMERA_BOUNDS: f32 = 8.0;
        let camera_y = OverworldCamera::compute_camera_y(CAMERA_BOUNDS);
        let aspect = DESIGN_H / DESIGN_W / 2.0;
        let horizon_world_y = camera_y + (1.0 / 3.0) * CAMERA_BOUNDS * aspect;
        assert!((horizon_world_y).abs() < 0.001);
    }
}
