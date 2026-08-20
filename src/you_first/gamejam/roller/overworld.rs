use glam::{Quat, Vec2, Vec3, Vec4};
use winit::keyboard::KeyCode;

use std::any::Any;
use std::fs;

use wgpu::WriteOnly;
use zerocopy::IntoBytes as _;

use crate::assets::SpriteEntry;
use crate::brushes::Brush;
use crate::demo::canvasing::spritecanvas::{BasicSpriteCanvas, SpriteCanvasSolver, SpriteInstance};
use crate::gamescope::scene::{Scene, SceneContext};
use crate::gather::impls::gather_ref;
use crate::gpuscope::canvasing::{CanvasTrait, CanvasUnderstander};
use crate::input::AxisConfig;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::ecs::scope::Scope;
use crate::ecs::{CanvasId, CanvasSolverId, MaterialId, PipId};
use crate::you_first::gamejam::roller::brush_flip::BrushFlipSolver;
use crate::you_first::gamejam::roller::bundles::player_roller_bundle;
use crate::you_first::gamejam::roller::camera::CameraDirector;
use crate::you_first::gamejam::roller::clouds::CloudDriftSystem;
use crate::you_first::gamejam::roller::components::RollerAddition;
use crate::you_first::gamejam::roller::controller::PlayerLateralController;

use crate::you_first::gamejam::roller::solver::RollerProjectionSolver;
use crate::you_first::gamejam::roller::spawner::RollerSpawner;

const DESIGN_W: f32 = 1280.0;
const DESIGN_H: f32 = 720.0;
const HORIZON_NDC_Y: f32 = 1.0 / 3.0;
// Visible world width for the roller; the world height is this divided by DESIGN_W/DESIGN_H.
const CAMERA_BOUNDS: f32 = 8.0 * (DESIGN_W / DESIGN_H);
const CAMERA_MARGIN: f32 = 3.0;

const WALK_HORIZON_Y: f32 = 0.0;
const GROUND_STRIP_HEIGHT: f32 = 20.0;
const TILTED_GROUND_HEIGHT: f32 = 1.4;
const GROUND_TILT_DEG: f32 = 5.0;

fn sprite_material_config(name: &str) -> (bool, Vec3) {
    match name {
        "cactus" => (true, Vec3::new(0.0, -0.07421875, 0.0)),
        "grass" => (true, Vec3::new(0.0, -0.048828125, 0.0)),
        "crate" => (true, Vec3::new(0.0, -0.038828125, 0.0)),
        "building_right" | "building_left" => (true, Vec3::new(0.0, -0.49625, 0.0)),
        "tumbleweed"
        | "player_fwd"
        | "player_fwd_old"
        | "player_happy"
        | "player_title"
        | "cactus_hourglass"
        | "grass_tiny"
        | "townie_1"
        | "townie_2"
        | "bandit-1"
        | "bandit-2"
        | "hat_good"
        | "hat_bad"
        | "bad_aim"
        | "bad_guys_hat_icon" => (true, Vec3::ZERO),
        "sun" | "cloud_1" | "cloud_2" => (false, Vec3::ZERO),
        _ => (false, Vec3::ZERO),
    }
}

#[derive(Debug)]
pub struct OverworldScene {
    state: State,
    sprites_dir: String,
    pixels_per_unit: f32,
    player: Option<PipId>,
    camera_director: CameraDirector,
    ground: Option<PipId>,
    tilted_ground: Option<PipId>,
    sky: Option<PipId>,
    sun: Option<PipId>,
}

#[derive(Debug)]
enum State {
    Setup,
    Running,
}

impl OverworldScene {
    pub fn new(pixels_per_unit: f32) -> Self {
        Self {
            state: State::Setup,
            sprites_dir: "assets/sprites".to_string(),
            pixels_per_unit,
            player: None,
            camera_director: CameraDirector::new(Vec2::ZERO, 10.0 / CAMERA_BOUNDS, 8.0, 8.0),
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

        let solver_id = ctx.gpu
            .canvas_renderer_mut()
            .solvers
            .insert(Box::new(SpriteCanvasSolver::new()));

        self.make_canvas(ctx, solver_id);
        ctx.solvers.register(RollerProjectionSolver);
        ctx.solvers.register(BrushFlipSolver);

        let player = ctx
            .asset_registry
            .get("player_fwd")
            .unwrap_or(ctx.asset_registry.get("__white__").unwrap())
            .clone();
        let player_id = self.spawn_player(ctx, &player);

        self.player = Some(player_id);

        let ground_z = 0.25 + (0.5 * 0.663_968);
        let ground_sprite = ctx
            .asset_registry
            .get("ground")
            .unwrap_or(ctx.asset_registry.get("__white__").unwrap());
        let ground = ctx.domain.make(|scope: &mut Scope| {
            let mut brush = Brush::new(ground_sprite.canvas, ground_sprite.material);
            brush.scale = Vec3::new(
                20.0 / ground_sprite.natural_scale.x,
                GROUND_STRIP_HEIGHT / ground_sprite.natural_scale.y,
                1.0,
            );
            brush.color = Vec4::new(0.25, 0.2, 0.15, 1.0);
            scope.core.with(
                Transform {
                    xyz: Vec3::new(0.0, WALK_HORIZON_Y - GROUND_STRIP_HEIGHT * 0.5, ground_z),
                    rot: Quat::IDENTITY,
                },
                brush,
                "ground".to_string(),
                Motion::default(),
            );
        });
        self.ground = Some(ground);

        let tilt_rad = GROUND_TILT_DEG.to_radians();
        let half_h = TILTED_GROUND_HEIGHT * 0.5;
        let tilted_z = ground_z - half_h * tilt_rad.sin();
        let tilted_y = WALK_HORIZON_Y - half_h * tilt_rad.cos();
        let tilted_sprite = ctx
            .asset_registry
            .get("tilted_ground")
            .unwrap_or(ctx.asset_registry.get("__white__").unwrap());
        let tilted = ctx.domain.make(|scope: &mut Scope| {
            let mut brush = Brush::new(tilted_sprite.canvas, tilted_sprite.material);
            brush.scale = Vec3::new(
                20.0 / tilted_sprite.natural_scale.x,
                TILTED_GROUND_HEIGHT / tilted_sprite.natural_scale.y,
                1.0,
            );
            brush.color = Vec4::new(0.25, 0.2, 0.15, 1.0);
            scope.core.with(
                Transform {
                    xyz: Vec3::new(0.0, tilted_y, tilted_z),
                    rot: Quat::from_rotation_x(tilt_rad),
                },
                brush,
                "tilted_ground".to_string(),
                Motion::default(),
            );
        });
        self.tilted_ground = Some(tilted);

        let sky_sprite = ctx
            .asset_registry
            .get("sky")
            .unwrap_or(ctx.asset_registry.get("__white__").unwrap());
        let sky = ctx.domain.make(|scope: &mut Scope| {
            let mut brush = Brush::new(sky_sprite.canvas, sky_sprite.material);
            brush.scale = Vec3::new(
                20.0 / sky_sprite.natural_scale.x,
                10.0 / sky_sprite.natural_scale.y,
                1.0,
            );
            brush.color = Vec4::new(0.6, 0.75, 0.9, 1.0);
            scope.core.with(
                Transform {
                    xyz: Vec3::new(0.0, WALK_HORIZON_Y + 5.0, 0.95),
                    rot: Quat::IDENTITY,
                },
                brush,
                "sky".to_string(),
                Motion::default(),
            );
        });
        self.sky = Some(sky);

        let sun_sprite = ctx
            .asset_registry
            .get("sun")
            .unwrap_or(ctx.asset_registry.get("__white__").unwrap());
        let sun = ctx.domain.make(|scope: &mut Scope| {
            let mut brush = Brush::new(sun_sprite.canvas, sun_sprite.material);
            brush.scale = Vec3::new(
                0.25 / sun_sprite.natural_scale.x,
                0.25 / sun_sprite.natural_scale.y,
                1.0,
            );
            brush.color = Vec4::new(1.0, 1.0, 0.6, 1.0);
            scope.core.with(
                Transform {
                    xyz: Vec3::new(-0.3, 0.9, 0.93),
                    rot: Quat::IDENTITY,
                },
                brush,
                "sun".to_string(),
                Motion::default(),
            );
        });
        self.sun = Some(sun);

        ctx.input.add_axis(
            "Horizontal",
            AxisConfig::new(
                vec![KeyCode::ArrowRight, KeyCode::KeyD],
                vec![KeyCode::ArrowLeft, KeyCode::KeyA],
            ),
        );
        ctx.input.add_axis(
            "Vertical",
            AxisConfig::new(
                vec![KeyCode::ArrowUp, KeyCode::KeyW],
                vec![KeyCode::ArrowDown, KeyCode::KeyS],
            ),
        );

        ctx.solvers
            .register(PlayerLateralController::new(player_id));
        ctx.solvers.register(RollerSpawner::new(player_id));
        ctx.solvers.register(CloudDriftSystem::new());
    }

    fn make_canvas(&mut self, ctx: &mut SceneContext, solver_id: CanvasSolverId) {
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

        let mut pending: Vec<(String, MaterialId, Vec2)> = Vec::new();
        let world_billboard_material;

        {
            let texture_scope = &mut gpu_context.texture_scope;

            let white_pixel = texture_scope.white_pixel(&gpu_context.device, &gpu_context.queue);
            let white_material = canvas
                .add_sprite(
                    &gpu_context.device,
                    &gpu_context.queue,
                    &mut every,
                    texture_scope,
                    white_pixel,
                    false,
                )
                .expect("failed to add white pixel sprite");
            world_billboard_material = canvas
                .add_sprite(
                    &gpu_context.device,
                    &gpu_context.queue,
                    &mut every,
                    texture_scope,
                    white_pixel,
                    true,
                )
                .expect("failed to add world billboard sprite");
            let white_scale = Vec2::ONE / self.pixels_per_unit;

            pending.push((
                "ground".to_string(),
                white_material,
                white_scale
            ));
            pending.push((
                "tilted_ground".to_string(),
                white_material,
                white_scale
            ));
            pending.push((
                "sky".to_string(),
                white_material,
                white_scale
            ));
            pending.push((
                "sun".to_string(),
                white_material,
                white_scale
            ));
            pending.push((
                "__white__".to_string(),
                white_material,
                white_scale
            ));
            pending.push((
                "world_rect".to_string(),
                world_billboard_material,
                white_scale
            ));
            pending.push((
                "world_billboard".to_string(),
                world_billboard_material,
                white_scale
            ));

            if let Ok(entries) = fs::read_dir(&self.sprites_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("png") {
                        continue;
                    }
                    let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                        continue;
                    };
                    let Some(img_id) = texture_scope.load_image(
                        &gpu_context.device,
                        &gpu_context.queue,
                        path.to_str().unwrap_or(""),
                    ) else {
                        log::warn!("failed to load sprite {}", path.display());
                        continue;
                    };
                    let (billboard, _) = sprite_material_config(name);
                    let Some(material) = canvas.add_sprite(
                        &gpu_context.device,
                        &gpu_context.queue,
                        &mut every,
                        texture_scope,
                        img_id,
                        billboard,
                    ) else {
                        log::warn!("failed to add sprite {} to canvas", path.display());
                        continue;
                    };
                    let Some((w, h)) = texture_scope.size(img_id) else {
                        continue;
                    };
                    let natural_scale = Vec2::new(w as f32, h as f32) / self.pixels_per_unit;
                    pending.push((
                        name.to_string(),
                        material,
                        natural_scale
                    ));
                }
            }
        }

        let canvas_id = ctx
            .gpu
            .canvas_renderer_mut()
            .canvases
            .insert((every, Box::new(canvas)));

        let spritesolver_dyn = ctx.gpu.canvas_renderer_mut().solvers.get_mut(solver_id)
        .expect("Requires supporting SpriteCanvasSolver");
        let spritesolver_ptr = spritesolver_dyn.as_mut();
        let spritesolver_any = spritesolver_ptr as &mut dyn Any;
        let spritesolver = spritesolver_any.downcast_mut::<SpriteCanvasSolver>()
        .expect("Requires supporting SpriteCanvasSolver");

        spritesolver.understanders.insert(
            canvas_id,
            Box::new(|_id, slice: &[((Transform, Brush), MaterialId, CanvasId)], out: WriteOnly<[u8]>, _canvas: &dyn CanvasTrait| {
                let mut written = 0;
                let mut out = out;
                for ((xform, brush), _material, _canvas_id) in slice {
                    let instance = SpriteInstance {
                        position: [xform.xyz.x, xform.xyz.y, xform.xyz.z],
                        rotation: [
                            xform.rot.x as f16,
                            xform.rot.y as f16,
                            xform.rot.z as f16,
                            xform.rot.w as f16,
                        ],
                        scale: [brush.scale.x as f16, brush.scale.y as f16, 0.0f16, 0.0f16],
                        color: [
                            brush.color.x as f16,
                            brush.color.y as f16,
                            brush.color.z as f16,
                            brush.color.w as f16,
                        ],
                    };
                    let bytes = instance.as_bytes();
                    let mut chunk = out.slice(written..written + bytes.len());
                    chunk.copy_from_slice(bytes);
                    written += bytes.len();
                }
                written
            }),
        );

        for (name, material, natural_scale) in pending {
            ctx.asset_registry.insert(
                name,
                SpriteEntry {
                    canvas: canvas_id,
                    material,
                    natural_scale,
                },
            );
        }
    }

    fn update_camera(&mut self, ctx: &mut SceneContext) {
        let aspect = DESIGN_H / DESIGN_W / 2.0;
        let camera_y = -HORIZON_NDC_Y * CAMERA_BOUNDS * aspect;
        let zoom = 10.0 / CAMERA_BOUNDS;

        let current_x = ctx.camera.pos.x;
        let target_x = if let Some(player) = self.player
            && let Some(xform) = gather_ref(&ctx.domain.ids, &ctx.domain.tables.core.xforms, player)
        {
            let player_x = xform.xyz.x;
            let delta = player_x - current_x;
            if delta > CAMERA_MARGIN {
                player_x - CAMERA_MARGIN
            } else if delta < -CAMERA_MARGIN {
                player_x + CAMERA_MARGIN
            } else {
                current_x
            }
        } else {
            current_x
        };

        self.camera_director
            .set_pan_target(Vec2::new(target_x, camera_y));
        self.camera_director.set_zoom_target(zoom);
        self.camera_director.update(ctx.camera, ctx.dt);
    }

    fn spawn_player(
        &mut self,
        ctx: &mut SceneContext,
        sprite: &SpriteEntry,
    ) -> PipId {
        ctx.domain.make(player_roller_bundle(
            sprite.canvas,
            sprite.material,
            0.0,
            2.0,
            Vec2::splat(1.28125),
            sprite.natural_scale,
            1.0,
        ))
    }

}

impl<T: 'static, F: 'static> CanvasUnderstander<T> for F
where
    F: for<'a> FnMut(CanvasId, &'a [(T, MaterialId, CanvasId)], WriteOnly<'a, [u8]>, &'a dyn CanvasTrait) -> usize,
{
    fn understand<'a>(
        &mut self,
        id: CanvasId,
        t: &'a [(T, MaterialId, CanvasId)],
        out: WriteOnly<'a, [u8]>,
        canvas: &'a dyn CanvasTrait,
    ) -> usize {
        self(id, t, out, canvas)
    }
}
