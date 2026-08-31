use glam::{Vec2, Vec3, Vec4};
use wgpu::TextureFormat;

use crate::addition::Addition;
use crate::brushes::Brush;
use crate::demo::canvasing::spritecanvas::{
    BasicSpriteCanvas, BasicSpriteCanvasUnderstander, SpriteCanvasSolver,
};
use crate::diagnostics::DiagnosticsAdd;
use crate::ecs::scope::Scope;
use crate::ecs::{CanvasId, MaterialId};
use crate::physics::PhysicsAdd;
use crate::spacial::boundary::Boundary;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;

use super::{Scene, SceneContext};

const COLUMNS: usize = 32;
const ROWS: usize = 64;
const SQUARE_COUNT: usize = COLUMNS * ROWS;
const PIXELS_PER_UNIT: f32 = 1.0;
const ZOOM: f32 = 0.08;
const RESTITUTION: f32 = 0.95;
const BOUNDARY_HALF: f32 = 35.0;
const BOUNDARY_Z: f32 = 1.0;

#[derive(Debug)]
pub struct TestScene {
    content: bool,
}

impl TestScene {
    pub fn new() -> Self {
        Self { content: false }
    }
}

impl Scene for TestScene {
    fn update(&mut self, ctx: &mut SceneContext) {
        if self.content {
            return;
        }
        self.content = true;
        setup(ctx);
    }
}

fn setup(ctx: &mut SceneContext) {
    ctx.domain
        .add::<PhysicsAdd>()
        .expect("PhysicsAdd must be available");
    ctx.camera.zoom = ZOOM;
    ctx.camera.pos = Vec2::ZERO;

    if let Some(signals) = DiagnosticsAdd::signals(&mut ctx.domain.signals) {
        signals.enabled = true;
    }

    ctx.domain.signals.core.boundary = Boundary {
        min: Vec3::new(-BOUNDARY_HALF, -BOUNDARY_HALF, -BOUNDARY_Z),
        max: Vec3::new(BOUNDARY_HALF, BOUNDARY_HALF, BOUNDARY_Z),
        restitution: RESTITUTION,
    };

    let (canvas_id, material) = make_canvas(ctx);
    spawn_squares(ctx, canvas_id, material);
}

fn make_canvas(ctx: &mut SceneContext) -> (CanvasId, MaterialId) {
    let format = ctx.gpu.surface_cfg().format;
    let sample_count = ctx.gpu.settings().sample_count;
    let depth_format = ctx
        .gpu
        .targets()
        .depth_enabled()
        .then_some(TextureFormat::Depth32Float);
    let dvc = &mut ctx.gpu.device;
    let img_id = dvc.texture_scope.white_pixel(&dvc.device, &dvc.queue);
    let (mut every, mut sprite_canvas) =
        BasicSpriteCanvas::make(&dvc.device, format, sample_count, depth_format, PIXELS_PER_UNIT);
    let material = sprite_canvas
        .add_sprite(
            &dvc.device,
            &dvc.queue,
            &mut every,
            &dvc.texture_scope,
            img_id,
            false,
            0.0,
        )
        .unwrap();
    let canvas_renderer = ctx.gpu.canvas_renderer_mut();
    let canvas_id = canvas_renderer
        .canvases
        .insert((every, Box::new(sprite_canvas)));
    let mut solver = SpriteCanvasSolver::new();
    solver
        .understanders
        .insert(canvas_id, Box::new(BasicSpriteCanvasUnderstander));
    canvas_renderer.solvers.insert(Box::new(solver));
    (canvas_id, material)
}

fn spawn_squares(ctx: &mut SceneContext, canvas_id: CanvasId, material: MaterialId) {
    for i in 0..SQUARE_COUNT {
        let x = (i % COLUMNS) as f32 - (COLUMNS / 2) as f32;
        let y = (i / COLUMNS) as f32 - (ROWS / 2) as f32;
        let transform = Transform {
            xyz: Vec3::new(x, y, 0.0),
            rot: glam::Quat::IDENTITY,
        };
        let mut brush = Brush::new(canvas_id, material);
        brush.scale = Vec3::splat(1.0);
        brush.color = Vec4::ONE;
        let motion = Motion::random_unit();

        ctx.domain.make(|scope: &mut Scope| {
            scope
                .core
                .with(transform, brush, motion);
            scope
                .view::<PhysicsAdd>()
                .expect("PhysicsAdd must be present")
                .with(1.0, Vec3::ZERO);
        });
    }
}
