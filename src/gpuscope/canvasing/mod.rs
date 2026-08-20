use std::any::Any;
use std::fmt::Debug;
use std::num::NonZero;

use slotmap::{SecondaryMap, SlotMap};
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, BufferViewMut, CommandEncoder, Device,
    RenderPass, RenderPipeline, WriteOnly,
};

use crate::spacial::camera::Camera;
use crate::ecs::tables::Tables;
use crate::ecs::{CanvasId, CanvasSolverId, MaterialId};

pub trait CanvasSolver: Any + std::fmt::Debug {
    fn solve(
        &mut self,
        tables: &Tables,
        view: &mut BufferViewMut,
        sink: &mut DrawWriter,
    ) -> usize;
}
pub trait CanvasUnderstander<T> {
    fn understand<'a>(
        &mut self,
        id: CanvasId,
        t: &'a [(T, MaterialId, CanvasId)],
        out: WriteOnly<'a, [u8]>,
        canvas: &'a dyn CanvasTrait,
    ) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct Draw {
    pub adr: u32,
    pub count: u32,
}

#[derive(Debug)]
pub struct EveryCanvas {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: RenderPipeline,
    pub material_ids: SlotMap<MaterialId, ()>,
    pub draws: SecondaryMap<MaterialId, Draw>,
}

impl EveryCanvas {
    pub fn new(bind_group_layout: BindGroupLayout, pipeline: RenderPipeline) -> Self {
        Self {
            bind_group_layout,
            pipeline,
            material_ids: SlotMap::with_key(),
            draws: SecondaryMap::new(),
        }
    }
}

pub trait CanvasTrait: Any + std::fmt::Debug {
    fn prepare(
        &mut self,
        camera: &Camera,
        encoder: &mut CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
    );
    fn begin_render_pass(&self, _pass: &mut RenderPass<'_>, _every: &EveryCanvas) {}
    fn render(
        &self,
        pass: &mut RenderPass<'_>,
        material: MaterialId,
        instances: std::ops::Range<u32>,
        every: &EveryCanvas,
    );
}

pub struct DrawWriter<'a> {
    canvases: &'a mut SlotMap<CanvasId, (EveryCanvas, Box<dyn CanvasTrait>)>,
    next_byte: u64,
}

impl DrawWriter<'_> {
    /// Reserve a byte range for `count` instances of size `instance_size`.
    /// Returns the instance start address and the byte offset at which to write.
    pub fn reserve(&mut self, count: u32, instance_size: u64) -> (u32, u64) {
        let aligned = self.next_byte.next_multiple_of(instance_size);
        let adr = aligned / instance_size;
        self.next_byte = aligned + u64::from(count) * instance_size;
        (adr as u32, aligned)
    }
    pub fn set_draw(&mut self, canvas: CanvasId, material: MaterialId, adr: u32, count: u32) {
        if let Some((every, _)) = self.canvases.get_mut(canvas) {
            every.draws.insert(material, Draw { adr, count });
        }
    }
    pub fn get_canvas(&self, canvas: CanvasId) -> Option<&dyn CanvasTrait> {
        self.canvases
            .get(canvas)
            .map(|(_, canvas)| canvas.as_ref())
    }
    pub fn bytes_used(&self) -> u64 {
        self.next_byte
    }
}

#[derive(Debug)]
pub struct CanvasRenderer {
    pub solvers: SlotMap<CanvasSolverId, Box<dyn CanvasSolver>>,
    pub canvases: SlotMap<CanvasId, (EveryCanvas, Box<dyn CanvasTrait>)>,

    //todo, gift to gpu-mem mod
    staging_belt: wgpu::util::StagingBelt,
    //todo, get a transient frame buffer from the lord
    instance_buffer: Buffer,
}

impl CanvasRenderer {
    pub fn make(device: Device) -> Self {
        const INSTANCE_BUFFER_BYTES: u32 = 1024 * 1024; //1MiB

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("canvas instances"),
            size: u64::from(INSTANCE_BUFFER_BYTES),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // The staging belt chunk must be at least as large as the largest single
        // write_buffer call. We open the full instance buffer once per frame.
        let belt = u64::from(INSTANCE_BUFFER_BYTES);

        let staging_belt = wgpu::util::StagingBelt::new(device, belt);

        Self {
            solvers: SlotMap::with_key(),
            canvases: SlotMap::with_key(),
            staging_belt,
            instance_buffer,
        }
    }

    pub fn prepare(&mut self, tables: &Tables, camera: &Camera, encoder: &mut CommandEncoder) {
        for (every, canvas) in self.canvases.values_mut() {
            every.draws.clear();
            canvas.prepare(camera, encoder, &mut self.staging_belt);
        }

        let instance_buffer = &self.instance_buffer;
        let total_bytes = NonZero::new(instance_buffer.size()).unwrap();
        let mut view = self
            .staging_belt
            .write_buffer(encoder, instance_buffer, 0, total_bytes);

        let canvases = &mut self.canvases;
        let solvers = &mut self.solvers;
        let mut writer = DrawWriter {
            canvases,
            next_byte: 0,
        };
        for (_id, solver) in solvers.iter_mut() {
            solver.as_mut().solve(tables, &mut view, &mut writer);
        }

        assert!(
            writer.bytes_used() <= instance_buffer.size(),
            "CanvasRenderer overran the instance buffer"
        );

        drop(view);
        self.staging_belt.finish();
    }

    pub fn render(&self, pass: &mut RenderPass<'_>) {
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        for (_canvas_id, (every, canvas)) in &self.canvases {
            canvas.begin_render_pass(pass, every);
            for (material, draw) in &every.draws {
                canvas.render(pass, material, draw.adr..draw.adr + draw.count, every);
            }
        }
    }

    pub fn recall(&mut self) {
        self.staging_belt.recall();
    }
}
