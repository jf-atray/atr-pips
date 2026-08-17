use glam::{Quat, Vec2, Vec3};

use std::fs;

use crate::assets::{SpriteEntry, SpriteRect};
use crate::brushes::Brush;
use crate::demo::canvasing::spritecanvas::{BasicSpriteCanvas, SpriteCanvasSolver};
use crate::gamescope::scene::{Scene, SceneContext};
use crate::gather::impls::gather_ref;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::scope::Scope;
use crate::tables::{CanvasId, MaterialId, PipId};
use crate::you_first::gamejam::roller::brush_flip::BrushFlipSolver;
use crate::you_first::gamejam::roller::bundles::{player_roller_bundle, roller_sprite_bundle};
use crate::you_first::gamejam::roller::components::RollerAddition;
use crate::you_first::gamejam::roller::projection::FAR_Z;
use crate::you_first::gamejam::roller::solver::RollerProjectionSolver;


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
    sprites_dir: String,
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
            sprites_dir: "assets/sprites".to_string(),
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

        self.make_canvas(ctx);
        ctx.gpu
            .canvas_renderer_mut()
            .solvers
            .insert(Box::new(SpriteCanvasSolver::new()));
        ctx.solvers.register(RollerProjectionSolver);
        ctx.solvers.register(BrushFlipSolver);

        let player = ctx.asset_registry.get("player_happy");
        let cactus = ctx.asset_registry.get("cactus");
        let canvas = player.canvas;

        self.player = Some(self.spawn_player(ctx, canvas, player.material));
        self.spawn_objects(ctx, canvas, cactus.material);

        let ground_z = 0.25 + (0.5 * 0.66396803);
        let ground_sprite = ctx.asset_registry.get("ground");
        let ground = ctx.domain.make(|scope: &mut Scope| {
            let mut brush = Brush::new(ground_sprite.canvas, ground_sprite.material);
            brush.scale = Vec3::new(20.0, GROUND_STRIP_HEIGHT, 1.0);
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
        let tilted_z = (ground_z - half_h * tilt_rad.sin());
        let tilted_y = WALK_HORIZON_Y - half_h * tilt_rad.cos();
        let tilted_sprite = ctx.asset_registry.get("tilted_ground");
        let tilted = ctx.domain.make(|scope: &mut Scope| {
            let mut brush = Brush::new(tilted_sprite.canvas, tilted_sprite.material);
            brush.scale = Vec3::new(20.0, TILTED_GROUND_HEIGHT, 1.0);
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

        let sky_sprite = ctx.asset_registry.get("sky");
        let sky = ctx.domain.make(|scope: &mut Scope| {
            let mut brush = Brush::new(sky_sprite.canvas, sky_sprite.material);
            brush.scale = Vec3::new(20.0, 10.0, 1.0);
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

        let sun_sprite = ctx.asset_registry.get("sun");
        let sun = ctx.domain.make(|scope: &mut Scope| {
            let mut brush = Brush::new(sun_sprite.canvas, sun_sprite.material);
            brush.scale = Vec3::new(0.25, 0.25, 1.0);
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

    }

    fn make_canvas(&mut self, ctx: &mut SceneContext) {
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

        let mut pending: Vec<(String, MaterialId, Vec2, SpriteRect)> = Vec::new();

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
                )
                .expect("failed to add white pixel sprite");
            pending.push((
                "ground".to_string(),
                white_material,
                Vec2::ONE,
                SpriteRect::full(1.0, 1.0),
            ));
            pending.push((
                "tilted_ground".to_string(),
                white_material,
                Vec2::ONE,
                SpriteRect::full(1.0, 1.0),
            ));
            pending.push((
                "sky".to_string(),
                white_material,
                Vec2::ONE,
                SpriteRect::full(1.0, 1.0),
            ));
            pending.push((
                "sun".to_string(),
                white_material,
                Vec2::ONE,
                SpriteRect::full(1.0, 1.0),
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
                    let Some(material) = canvas.add_sprite(
                        &gpu_context.device,
                        &gpu_context.queue,
                        &mut every,
                        texture_scope,
                        img_id,
                    ) else {
                        continue;
                    };
                    let Some((w, h)) = texture_scope.size(img_id) else {
                        continue;
                    };
                    let natural_scale = Vec2::new(w as f32, h as f32) / self.pixels_per_unit;
                    pending.push((
                        name.to_string(),
                        material,
                        natural_scale,
                        SpriteRect::full(w as f32, h as f32),
                    ));
                }
            }
        }

        let canvas_id = ctx
            .gpu
            .canvas_renderer_mut()
            .canvases
            .insert((every, Box::new(canvas)));

        for (name, material, natural_scale, rect) in pending {
            ctx.asset_registry.register(
                name,
                SpriteEntry {
                    canvas: canvas_id,
                    material,
                    natural_scale,
                    rect,
                },
            );
        }
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
        ctx.domain.make(player_roller_bundle(
            canvas,
            material,
            0.0,
            2.0,
            [255, 255, 255, 255],
            1.0,
        ))
    }

    fn spawn_objects(&mut self, ctx: &mut SceneContext, canvas: CanvasId, material: MaterialId) {
        for i in -2..=2 {
            let lateral = i as f32 * 1.5;
            ctx.domain.make(roller_sprite_bundle(
                canvas,
                material,
                lateral,
                FAR_Z,
                [255, 255, 255, 255],
                1.0,
                0.0,
                i % 2 == 0,
            ));
        }
    }
}
