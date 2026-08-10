use slotmap::{SecondaryMap, SlotMap};
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, RenderPass, RenderPipeline,
};

use crate::spacial::camera::Camera;
use crate::tables::{CanvasId, CanvasSolverId, MaterialId};
use crate::tables::tables::Tables;

pub trait CanvasSolver {
    fn solve(
        &mut self,
        tables: &Tables,
        encoder: &mut CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        instance_buffer: &Buffer,
        solver_id: CanvasSolverId,
        sink: &mut DrawWriter,
    );
}


#[derive(Debug, Clone, Copy)]
pub struct Draw {
    pub adr: u32,
    pub count: u32,
}


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

pub trait CanvasTrait {
    fn prepare(&mut self, camera: &Camera, encoder: &mut CommandEncoder, belt: &mut wgpu::util::StagingBelt);
    fn begin_render_pass(&self, _pass: &mut RenderPass<'_>, _every: &EveryCanvas) {}
    fn render(&self, pass: &mut RenderPass<'_>, material: MaterialId, instances: std::ops::Range<u32>, every: &EveryCanvas);
}


pub struct DrawWriter<'a> {
    canvases: &'a mut SlotMap<CanvasId, (EveryCanvas, Box<dyn CanvasTrait>)>,
    next_adr: u32,
}

impl<'a> DrawWriter<'a> {
    pub fn reserve(&mut self, count: u32) -> u32 {
        let adr = self.next_adr;
        self.next_adr += count;
        adr
    }
    pub fn set_draw(&mut self, canvas: CanvasId, material: MaterialId, adr: u32, count: u32) {
        if let Some((every, _)) = self.canvases.get_mut(canvas) {
            every.draws.insert(material, Draw { adr, count });
        }
    }
}

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
        const INSTANCE_BUFFER_BYTES: u64 = 1024 * 1024 * 1;

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("canvas instances"),
            size: INSTANCE_BUFFER_BYTES,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let staging_belt = wgpu::util::StagingBelt::new(device, INSTANCE_BUFFER_BYTES);
        
        Self {
            solvers: SlotMap::with_key(),
            canvases: SlotMap::with_key(),
            staging_belt,
            instance_buffer,
        }
    }

    pub fn prepare(
        &mut self,
        tables: &Tables,
        camera: &Camera,
        encoder: &mut CommandEncoder,
    ) {
        for (every, canvas) in self.canvases.values_mut() {
            every.draws.clear();
            canvas.prepare(camera, encoder, &mut self.staging_belt);
        }

        let instance_buffer = &self.instance_buffer;
        let staging = &mut self.staging_belt;
        let canvases = &mut self.canvases;
        let solvers = &mut self.solvers;
        let mut writer = DrawWriter { canvases, next_adr: 0 };
        for (id, solver) in solvers.iter_mut() {
            solver.as_mut().solve(
                tables,
                encoder,
                staging,
                instance_buffer,
                id,
                &mut writer,
            );
        }
        staging.finish();
    }

    pub fn render(&self, pass: &mut RenderPass<'_>) {
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        for (_canvas_id, (every, canvas)) in self.canvases.iter() {
            canvas.begin_render_pass(pass, every);
            for (material, draw) in every.draws.iter() {
                canvas.render(pass, material, draw.adr..draw.adr + draw.count, every);
            }
        }
    }

    pub fn recall(&mut self) {
        self.staging_belt.recall();
    }
}

