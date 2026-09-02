use std::collections::HashMap;

use glam::{Vec2, Vec3, Vec4};
use wgpu::TextureFormat;
use winit::keyboard::KeyCode;

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::brushes::Brush;
use crate::demo::canvasing::spritecanvas::{
    BasicSpriteCanvas, BasicSpriteCanvasUnderstander, SpriteCanvasSolver,
};
use crate::diagnostics::DiagnosticsAdd;
use crate::ecs::scope::Scope;
use crate::ecs::{CanvasId, MaterialId, PipId};
use crate::gamescope::camera::{CameraMode, PanSource, ZoomSource};
use crate::input::{AxisConfig, Input};
use crate::collision::CollisionAdd;
use crate::physics::PhysicsAdd;
use crate::physics::data::material::Material;
use crate::spacial::aabb::Aabb;
use crate::spacial::motion::{Motion, MotionKind};
use crate::spacial::transform::Transform;

use super::{Scene, SceneContext};

#[derive(Debug)]
pub struct ColorSolver;

impl Solver for ColorSolver {}

impl ColorSolver {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &mut self,
        _dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        _signals: &mut SignalsMap,
        _input: &mut Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let core = &mut pips.tables.core;
        for (class_id, motion_col) in core.motions.data.iter() {
            let Some(brush_col) = core.brushes.data.get_mut(class_id) else { continue };
            for i in 0..motion_col.len() {
                let brush = &mut brush_col[i];
                if motion_col.key == MotionKind::Sleeping {
                    brush.color = Vec4::ONE;
                } else if motion_col.key == MotionKind::Active {
                    let v = motion_col[i].vel;
                    brush.color = Vec4::new(
                        (v.x + 1.0) * 0.5,
                        (v.y + 1.0) * 0.5,
                        0.0,
                        1.0,
                    );
                }
            }
        }
    }
}

const COLUMNS: usize = 32;
const ROWS: usize = 64;
const SQUARE_COUNT: usize = COLUMNS * ROWS;
const PIXELS_PER_UNIT: f32 = 1.0;
const ZOOM: f32 = 125.0;
const BOUNDARY_HALF: f32 = 35.0;

#[derive(Debug)]
pub struct TestScene {
    content: bool,
    track_target: Option<PipId>,
}

impl TestScene {
    pub fn new() -> Self {
        Self {
            content: false,
            track_target: None,
        }
    }
}

impl Scene for TestScene {
    fn update(&mut self, ctx: &mut SceneContext) {
        if !self.content {
            self.content = true;
            setup(ctx, self);
        }

        if ctx.input.axes.delta("camera_mode_fixed") == Some(1) {
            *ctx.camera_mode = CameraMode {
                pan: PanSource::Fixed { value: ctx.camera.pos },
                zoom: ZoomSource::Fixed { value: ctx.camera.zoom },
            };
        } else if ctx.input.axes.delta("camera_mode_manual") == Some(1) {
            *ctx.camera_mode = CameraMode {
                pan: PanSource::Manual { look: 5.0 },
                zoom: ZoomSource::Manual { look: 5.0 },
            };
        } else if let Some(target) = self.track_target
            && ctx.input.axes.delta("camera_mode_track") == Some(1)
        {
            *ctx.camera_mode = CameraMode {
                pan: PanSource::Track { target },
                zoom: ZoomSource::Fixed { value: ZOOM },
            };
        }
    }
}

fn setup(ctx: &mut SceneContext, test: &mut TestScene) {
    ctx.domain
        .add::<PhysicsAdd>()
        .expect("PhysicsAdd must be available");
    ctx.domain
        .add::<CollisionAdd>()
        .expect("CollisionAdd must be available");
    ctx.domain
        .add::<TestAdd>()
        .expect("TestAdd must be available");
    ctx.camera.zoom = ZOOM;
    *ctx.camera_mode = CameraMode {
        pan: PanSource::Fixed { value: Vec2::ZERO },
        zoom: ZoomSource::Fixed { value: ZOOM },
    };

    ctx.input.add_axis(
        "camera_mode_fixed",
        AxisConfig::new(vec![KeyCode::KeyF], vec![]),
    );
    ctx.input.add_axis(
        "camera_mode_manual",
        AxisConfig::new(vec![KeyCode::KeyM], vec![]),
    );
    ctx.input.add_axis(
        "camera_mode_track",
        AxisConfig::new(vec![KeyCode::KeyT], vec![]),
    );

    if let Some(signals) = DiagnosticsAdd::signals(&mut ctx.domain.signals) {
        signals.enabled = true;
    }

    ctx.domain.signals.core.drag = 0.1;

    let (canvas_id, material) = make_canvas(ctx);
    test.track_target = spawn_squares(ctx, canvas_id, material);
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

fn spawn_squares(ctx: &mut SceneContext, canvas_id: CanvasId, material: MaterialId) -> Option<PipId> {
    let mut first = None;

    spawn_walls(ctx, canvas_id, material);

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

        let aabb = Aabb::from_center_extent(transform.xyz, brush.scale * 0.5);

        let pip = ctx.domain.make(|scope: &mut Scope| {
            scope
                .core
                .with(transform, brush, (motion, MotionKind::Active));
            scope
                .view::<PhysicsAdd>()
                .expect("PhysicsAdd must be present")
                .with(1.0, 6.0, Vec3::ZERO, Material { friction: 0.5, restitution: 0.3 });
            scope
                .view::<CollisionAdd>()
                .expect("CollisionAdd must be present")
                .with(aabb);
        });

        if i == 0 {
            first = Some(pip);
        }
    }
    first
}

fn spawn_walls(ctx: &mut SceneContext, canvas_id: CanvasId, material: MaterialId) {
    let thickness = 2.0;
    let half = BOUNDARY_HALF;
    let walls = [
        (Vec3::new(0.0, -half - thickness * 0.5, 0.0), Vec3::new(half * 2.0 + thickness * 2.0, thickness, 1.0)),
        (Vec3::new(0.0, half + thickness * 0.5, 0.0), Vec3::new(half * 2.0 + thickness * 2.0, thickness, 1.0)),
        (Vec3::new(-half - thickness * 0.5, 0.0, 0.0), Vec3::new(thickness, half * 2.0, 1.0)),
        (Vec3::new(half + thickness * 0.5, 0.0, 0.0), Vec3::new(thickness, half * 2.0, 1.0)),
    ];

    for (pos, scale) in walls {
        let transform = Transform { xyz: pos, rot: glam::Quat::IDENTITY };
        let mut brush = Brush::new(canvas_id, material);
        brush.scale = scale;
        brush.color = Vec4::new(0.3, 0.3, 0.3, 1.0);
        let aabb = Aabb::from_center_extent(transform.xyz, brush.scale * 0.5);

        ctx.domain.make(|scope: &mut Scope| {
            scope
                .core
                .with(transform, brush, (Motion::default(), MotionKind::Static));
            scope
                .view::<PhysicsAdd>()
                .expect("PhysicsAdd must be present")
                .with(0.0, 0.0, Vec3::ZERO, Material { friction: 0.5, restitution: 0.3 });
            scope
                .view::<CollisionAdd>()
                .expect("CollisionAdd must be present")
                .with(aabb);
        });
    }
}

crate::addition! {
    #[derive(Debug)]
    pub struct test_world : TestAdd {
        tables: (),
        solvers: { color: ColorSolver = ColorSolver::new() },
        scripts: {},
        signals: (),
    }
}
