use std::fs;

use glam::{Quat, Vec2, Vec3};
use rand::Rng;

use crate::assets::{SpriteEntry, SpriteRect};
use crate::brushes::Brush;
use crate::demo::canvasing::spritecanvas::{BasicSpriteCanvas, SpriteCanvasSolver};
use crate::gamescope::motion::MotionSolver;
use crate::gamescope::scene::{Scene, SceneContext};
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::scope::Scope;
use crate::tables::{CanvasId, MaterialId};

pub struct Demo {
    sprites_dir: String,
    pixels_per_unit: f32,
    state: State,
}

enum State {
    Setup,
    Running,
}

impl Demo {
    pub fn new(sprites_dir: impl Into<String>, pixels_per_unit: f32) -> Self {
        Self {
            sprites_dir: sprites_dir.into(),
            pixels_per_unit,
            state: State::Setup,
        }
    }
}

impl Scene for Demo {
    fn update(&mut self, ctx: &mut SceneContext) {
        if matches!(self.state, State::Setup) {
            self.boot(ctx);
            self.state = State::Running;
        }
    }
}

impl Demo {
    fn boot(&mut self, ctx: &mut SceneContext) {
        self.load_sprites(ctx);
        self.register_solvers(ctx);
        self.spawn_pips(ctx);
    }

    fn load_sprites(&mut self, ctx: &mut SceneContext) -> CanvasId {
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
                .add_sprite(&gpu_context.device, &gpu_context.queue, &mut every, texture_scope, white_pixel)
                .expect("failed to add white pixel sprite");
            pending.push((
                "green".to_string(),
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
                    let Some(img_id) =
                        texture_scope.load_image(&gpu_context.device, &gpu_context.queue, path.to_str().unwrap_or(""))
                    else {
                        log::warn!("failed to load sprite {}", path.display());
                        continue;
                    };
                    let Some(material) =
                        canvas.add_sprite(&gpu_context.device, &gpu_context.queue, &mut every, texture_scope, img_id)
                    else {
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

        canvas_id
    }

    fn register_solvers(&mut self, ctx: &mut SceneContext) {
        ctx.gpu
            .canvas_renderer_mut()
            .solvers
            .insert(Box::new(SpriteCanvasSolver::new()));
        ctx.solvers.register(MotionSolver::new());
    }

    fn spawn_pips(&mut self, ctx: &mut SceneContext) {
        let mut rng = rand::rng();
        if ctx.asset_registry.is_empty() {
            log::warn!("Demo: no sprites loaded, skipping pip spawn");
            return;
        }

        for _ in 0..20 {
            let name = ctx.asset_registry.pick_random(&mut rng).to_string();
            let sprite = *ctx.asset_registry.get(&name);

            let transform = Transform {
                xyz: Vec3::new(
                    rng.random_range(-4.0..4.0),
                    rng.random_range(-4.0..4.0),
                    0.0,
                ),
                rot: Quat::IDENTITY,
            };
            let brush = Brush::from_sprite(&sprite);

            ctx.domain.make(|scope: &mut Scope| {
                scope.core.with(transform, brush, name, Motion::random_unit());
            });
        }
    }
}


