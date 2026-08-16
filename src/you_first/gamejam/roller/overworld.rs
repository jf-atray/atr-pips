use glam::{Quat, Vec2, Vec3};

use crate::brushes::Brush;
use crate::demo::canvasing::spritecanvas::{BasicSpriteCanvas, SpriteCanvasSolver};
use crate::gamescope::scene::{Scene, SceneContext};
use crate::gather::impls::gather_ref;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::scope::Scope;
use crate::tables::{CanvasId, MaterialId, PipId};
use crate::you_first::gamejam::roller::components::{
    RollerAddition, RollerDepth, RollerPlayer, RollerView,
};
use crate::you_first::gamejam::roller::projection::{FAR_Z, WALK_SPEED};
use crate::you_first::gamejam::roller::solver::RollerProjectionSolver;

use crate::gamejam::bundles::SpriteBundle;
use crate::gamejam::roller::bundle::PlayerRollerBundle;
use crate::gamejam::roller::camera::OverworldCamera;
use crate::gamejam::roller::system::{
    CloudDriftSystem, PlayerLateralController, RollerProjectionSystem, RollerSpawner,
};

const DESIGN_W: f32 = 1280.0;
const DESIGN_H: f32 = 720.0;
const HORIZON_NDC_Y: f32 = 1.0 / 3.0;
const CAMERA_BOUNDS: f32 = 8.0;
const CAMERA_MARGIN: f32 = 3.0;

const WALK_HORIZON_Y: f32 = 0.0;
const GROUND_STRIP_HEIGHT: f32 = 20.0;
const TILTED_GROUND_HEIGHT: f32 = 1.4;
const GROUND_TILT_DEG: f32 = 5.0;

pub struct OverworldScene {
    state: State,
    pixels_per_unit: f32,
    player: Option<PipId>,
    camera_x: f32,
    ground: Option<PipId>,
    tilted_ground: Option<PipId>,
    sky: Option<PipId>,
    sun: Option<PipId>,
}

enum State {
    Setup,
    Running,
}

impl OverworldScene {
    pub fn new(pixels_per_unit: f32) -> Self {
        Self {
            state: State::Setup,
            pixels_per_unit,
            player: None,
            camera_x: 0.0,
            ground: None,
            tilted_ground: None,
            sky: None,
            sun: None,
        }
    }
}

impl Scene for OverworldScene {
    fn update(&mut self, ctx: &mut SceneContext) {
        if matches!(self.state, State::Setup) {
            self.boot(ctx);
            self.state = State::Running;
        }
        self.update_camera(ctx);
    }
}

impl OverworldScene {
    fn boot(&mut self, ctx: &mut SceneContext) {
        ctx.domain.tables.add(RollerAddition::new());

        let (canvas, player_mat, cactus_mat) = self.make_canvas(ctx);
        ctx.gpu
            .canvas_renderer_mut()
            .solvers
            .insert(Box::new(SpriteCanvasSolver::new()));
        ctx.solvers.register(RollerProjectionSolver);

        self.player = Some(self.spawn_player(ctx, canvas, player_mat));
        self.spawn_objects(ctx, canvas, cactus_mat);

        let world = &mut ctx.world;
        let resources = &ctx.resources;
        let player = self.player.unwrap();

        let ground = SpriteBundle::from_asset(
            Vec3::new(0.0, WALK_HORIZON_Y - GROUND_STRIP_HEIGHT * 0.5, 0.25 + (0.5 * 0.66396803) ),
            Vec2::new(20.0, GROUND_STRIP_HEIGHT),
            [0.25, 0.2, 0.15, 1.0],
            "ground",
            resources,
        )
        .spawn(world);
        self.ground = Some(ground);

        let ground_z = 0.25 + (0.5 * 0.66396803);
        let tilt_rad = GROUND_TILT_DEG.to_radians();
        let half_h = TILTED_GROUND_HEIGHT * 0.5;
        let tilted_z = (ground_z - half_h * tilt_rad.sin());
        let tilted_y = WALK_HORIZON_Y - half_h * tilt_rad.cos();
        let mut tilted = SpriteBundle::from_asset(
            Vec3::new(0.0, tilted_y, tilted_z),
            Vec2::new(20.0, TILTED_GROUND_HEIGHT),
            [0.25, 0.2, 0.15, 1.0],
            "tilted_ground",
            resources,
        );
        tilted.transform.rot = Quat::from_rotation_x(tilt_rad);
        let tilted = tilted.spawn(world);
        self.tilted_ground = Some(tilted);

        let sky = SpriteBundle::from_asset(
            Vec3::new(0.0, WALK_HORIZON_Y + 5.0, 0.95),
            Vec2::new(20.0, 10.0),
            [0.6, 0.75, 0.9, 1.0],
            "sky",
            resources,
        )
        .spawn(world);
        self.sky = Some(sky);

        let sun = SpriteBundle::from_asset(
            Vec3::new(-0.3, 0.9, 0.93),
            Vec2::new(0.25, 0.25),
            [1.0, 1.0, 0.6, 1.0],
            "sun",
            resources,
        )
        .spawn(world);
        self.sun = Some(sun);

        ctx.scripts.register(Box::new(OverworldCamera::new(player)));
        ctx.scripts.register(Box::new(RollerProjectionSystem::new(player)));
        ctx.scripts.register(Box::new(PlayerLateralController::new(player)));
        ctx.scripts.register(Box::new(RollerSpawner::new(player)));
        ctx.scripts.register(Box::new(CloudDriftSystem::new()));
    }

    fn make_canvas(&mut self, ctx: &mut SceneContext) -> (CanvasId, MaterialId, MaterialId) {
        let gpu_context = &mut ctx.gpu.device;
        let format = ctx.gpu.surface.cfg.format;
        let sample_count = ctx.gpu.targets.sample_count();
        let depth_format = ctx
            .gpu
            .targets
            .depth_enabled()
            .then_some(wgpu::TextureFormat::Depth32Float);

        let (mut every, mut canvas) = BasicSpriteCanvas::make(
            &gpu_context.device,
            format,
            sample_count,
            depth_format,
            self.pixels_per_unit,
        );

        let texture_scope = &mut gpu_context.texture_scope;
        let player_img = texture_scope
            .load_image(
                &gpu_context.device,
                &gpu_context.queue,
                "assets/sprites/player_happy.png",
            )
            .expect("failed to load player sprite");
        let cactus_img = texture_scope
            .load_image(
                &gpu_context.device,
                &gpu_context.queue,
                "assets/sprites/cactus.png",
            )
            .expect("failed to load cactus sprite");

        let player_mat = canvas
            .add_sprite(
                &gpu_context.device,
                &gpu_context.queue,
                &mut every,
                texture_scope,
                player_img,
            )
            .expect("failed to add player material");
        let cactus_mat = canvas
            .add_sprite(
                &gpu_context.device,
                &gpu_context.queue,
                &mut every,
                texture_scope,
                cactus_img,
            )
            .expect("failed to add cactus material");

        let canvas_id = ctx
            .gpu
            .canvas_renderer_mut()
            .canvases
            .insert((every, Box::new(canvas)));

        (canvas_id, player_mat, cactus_mat)
    }

    fn update_camera(&mut self, ctx: &mut SceneContext) {
        if let Some(player) = self.player
            && let Some(xform) = gather_ref(&ctx.domain.ids, &ctx.domain.tables.core.xforms, player)
        {
            let player_x = xform.xyz.x;
            let delta = player_x - self.camera_x;
            if delta > CAMERA_MARGIN {
                self.camera_x = player_x - CAMERA_MARGIN;
            } else if delta < -CAMERA_MARGIN {
                self.camera_x = player_x + CAMERA_MARGIN;
            }
        }

        let aspect = DESIGN_H / DESIGN_W / 2.0;
        let camera_y = -HORIZON_NDC_Y * CAMERA_BOUNDS * aspect;
        let zoom = 10.0 / CAMERA_BOUNDS;
        ctx.camera.pos = Vec2::new(self.camera_x, camera_y);
        ctx.camera.zoom = zoom;
    }

    fn spawn_player(
        &mut self,
        ctx: &mut SceneContext,
        canvas: CanvasId,
        material: MaterialId,
    ) -> PipId {
        let transform = Transform {
            xyz: Vec3::ZERO,
            rot: Quat::IDENTITY,
        };
        let brush = Brush::new(canvas, material);
        let name = "player".to_string();
        let roller_player = RollerPlayer {
            walk_distance: 0.0,
            lateral: 0.0,
        };
        let roller_depth = RollerDepth {
            d: 2.0,
            lateral: 0.0,
            speed: -WALK_SPEED,
            scalar: 1.0,
            lateral_speed: 0.0,
        };

        ctx.domain.make(|scope: &mut Scope| {
            scope.core.with(transform, brush, name, Motion::default());
            scope
                .view::<RollerView>()
                .map(|rv| rv.with(roller_depth, roller_player));
        })
    }

    fn spawn_objects(&mut self, ctx: &mut SceneContext, canvas: CanvasId, material: MaterialId) {
        for i in -2..=2 {
            let lateral = i as f32 * 1.5;
            let transform = Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            };
            let brush = Brush::new(canvas, material);
            let name = format!("roller_{}", i);
            let roller_depth = RollerDepth {
                d: FAR_Z,
                lateral,
                speed: 0.0,
                scalar: 1.0,
                lateral_speed: 0.0,
            };

            ctx.domain.make(|scope: &mut Scope| {
                scope.core.with(transform, brush, name, Motion::default());
                scope.view::<RollerView>().map(|rv| rv.roller_depths = Some(roller_depth));
            });
        }
    }
}
