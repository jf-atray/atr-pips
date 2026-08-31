use std::any::Any;
use std::fmt::Debug;
use std::mem::size_of;
use std::num::NonZero;

use bumpalo::Bump;
use slotmap::{Key, SecondaryMap, SlotMap};
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferSize, BufferUsages, BufferViewMut,
    CommandEncoder, Device, RenderPass, RenderPipeline, WriteOnly,
};
use zerocopy::IntoBytes as _;

use crate::spacial::camera::Camera;
use crate::addition::TablesMap;
use crate::ecs::{CanvasId, CanvasSolverId, MaterialId};

pub trait CanvasSolver: Any + std::fmt::Debug {
    fn solve(
        &mut self,
        tables: &mut TablesMap,
        view: &mut BufferViewMut,
        sink: &mut DrawWriter,
        arena: &mut Bump,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draw {
    pub adr: u32,
    pub count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, zerocopy::IntoBytes, zerocopy::Immutable)]
pub(crate) struct DrawIndirect {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

#[derive(Debug)]
pub struct EveryCanvas {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: RenderPipeline,
    pub material_ids: SlotMap<MaterialId, ()>,
    pub draws: SecondaryMap<MaterialId, Draw>,
    pub draw_buffer: Buffer,
    draw_buffer_size: u64,
    draw_buffer_zeros: Vec<u8>,
    last_draws: SecondaryMap<MaterialId, Draw>,
}

impl EveryCanvas {
    pub fn new(
        device: &Device,
        bind_group_layout: BindGroupLayout,
        pipeline: RenderPipeline,
        max_materials: u64,
    ) -> Self {
        let draw_buffer_size = max_materials * u64::try_from(size_of::<DrawIndirect>()).unwrap();
        let draw_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("canvas draw commands"),
            size: draw_buffer_size,
            usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            bind_group_layout,
            pipeline,
            material_ids: SlotMap::with_key(),
            draws: SecondaryMap::new(),
            draw_buffer,
            draw_buffer_size,
            draw_buffer_zeros: vec![0u8; usize::try_from(draw_buffer_size).unwrap()],
            last_draws: SecondaryMap::new(),
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
        material: &MaterialId,
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
        (
            u32::try_from(adr).expect("No support for sci-fi hardware"),
            aligned,
        )
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

    pub fn prepare(
        &mut self,
        tables: &mut TablesMap,
        camera: &Camera,
        encoder: &mut CommandEncoder,
        arena: &mut Bump,
    ) {
        for (every, canvas) in self.canvases.values_mut() {
            every.draws.clear();
            canvas.prepare(camera, encoder, &mut self.staging_belt);
        }

        let instance_buffer = &self.instance_buffer;
        let total_bytes = NonZero::new(instance_buffer.size()).unwrap();
        let mut view = self
            .staging_belt
            .write_buffer(encoder, instance_buffer, 0, BufferSize::new(total_bytes.get()).unwrap());

        {
            let canvases = &mut self.canvases;
            let solvers = &mut self.solvers;
            let mut writer = DrawWriter {
                canvases,
                next_byte: 0,
            };
            for (_id, solver) in solvers.iter_mut() {
                arena.reset();
                solver.as_mut().solve(tables, &mut view, &mut writer, arena);
                arena.reset();
            }

            assert!(
                writer.bytes_used() <= instance_buffer.size(),
                "CanvasRenderer overran the instance buffer"
            );
        }

        drop(view);

        for (canvas_id, (every, _canvas)) in self.canvases.iter_mut() {
            if every.draws == every.last_draws {
                continue;
            }

            log::info!("re-recording draw commands for canvas {canvas_id:?}");

            let mut draw_view = self
                .staging_belt
                .write_buffer(
                    encoder,
                    &every.draw_buffer,
                    0,
                    BufferSize::new(every.draw_buffer_size).unwrap(),
                );
            draw_view.copy_from_slice(&every.draw_buffer_zeros);

            for (material, draw) in every.draws.iter() {
                let cmd = DrawIndirect {
                    vertex_count: 6,
                    instance_count: draw.count,
                    first_vertex: 0,
                    first_instance: draw.adr,
                };
                let offset =
                    u64::from(material.data().as_ffi() as u32) * u64::try_from(size_of::<DrawIndirect>()).unwrap();
                let bytes = cmd.as_bytes();
                let offset_usize = usize::try_from(offset).unwrap();
                draw_view
                    .slice(offset_usize..offset_usize + bytes.len())
                    .copy_from_slice(bytes);
            }

            every.last_draws = every.draws.clone();
        }

        self.staging_belt.finish();
    }

    pub fn render(&self, pass: &mut RenderPass<'_>) {
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        for (_canvas_id, (every, canvas)) in &self.canvases {
            canvas.begin_render_pass(pass, every);
            for (material, draw) in every.draws.iter() {
                canvas.render(pass, &material, draw.adr..draw.adr + draw.count, every);
            }
        }
    }

    pub fn recall(&mut self) {
        self.staging_belt.recall();
    }
}
