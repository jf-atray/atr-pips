use std::iter;
use std::mem::size_of;
use zerocopy::IntoBytes as _;

use glam::{Quat, Vec3A};
use slotmap::SlotMap;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, RenderPass,
};

use crate::query::impls::query_ref_ref;
use crate::tables::{CanvasId, CanvasSolverId};
use crate::tables::demo::DemoTables;

pub trait CanvasSolver {
    fn instance_size(&self) -> u32;

    fn solve(
        &mut self,
        tables: &DemoTables,
        encoder: &mut CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        instance_buffer: &Buffer,
        solver_id: CanvasSolverId,
        adr: u32,
        draws: &mut Vec<CanvasDraw>,
    ) -> u32;
}


#[derive(Debug, Clone, Copy)]
pub struct CanvasDraw {
    pub canvas: CanvasId,
    pub adr: u32,
    pub count: u32,
}

pub struct CanvasRenderer {
    pub solvers: SlotMap<CanvasSolverId, Box<dyn CanvasSolver>>,

    //todo, gift to gpu-mem mod
    staging_belt: wgpu::util::StagingBelt,
    //todo, get a transient frame buffer from the lord
    instance_buffer: Buffer,
    draws: Vec<CanvasDraw>,
}

impl CanvasRenderer {
    pub fn make(device: Device) -> Self {
        const INSTANCE_BUFFER_BYTES: u64 = 1024 * 10;

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("canvas instances"),
            size: INSTANCE_BUFFER_BYTES,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let staging_belt = wgpu::util::StagingBelt::new(device, INSTANCE_BUFFER_BYTES);
        
        Self {
            solvers: SlotMap::with_key(),
            staging_belt,
            instance_buffer,
            draws: Vec::new(),
        }
    }

    pub fn prepare(
        &mut self,
        tables: &DemoTables,
        encoder: &mut CommandEncoder,
    ) -> &[CanvasDraw] {
        self.draws.clear();
        let mut adr: u32 = 0;
        for (id, solver) in self.solvers.iter_mut() {
            let count = solver.as_mut().solve(
                tables,
                encoder,
                &mut self.staging_belt,
                &self.instance_buffer,
                id,
                adr,
                &mut self.draws,
            );
            adr += count;
        }
        self.staging_belt.finish();
        &self.draws
    }

    pub fn render(&self, _pass: &mut RenderPass<'_>) {
        for _draw in &self.draws {
            // Set pipeline/bind group per (canvas, material), then draw(adr, count).
        }
    }

    pub fn recall(&mut self) {
        self.staging_belt.recall();
    }
}

pub struct SimpleCanvasSolver {
    sort: Vec<(SimpleCanvasInstance, CanvasId)>,
    instances: Vec<SimpleCanvasInstance>,
}

impl SimpleCanvasSolver {
    pub fn new() -> Self {
        Self {
            sort: Vec::new(),
            instances: Vec::new(),
        }
    }

    fn collect_from(&mut self, tables: &DemoTables, my_id: CanvasSolverId) {
        self.sort.clear();
        //our first real query!
        //next up, macro-assist this vec-of-vecs iteration
        for (xforms, brushes) in query_ref_ref(&tables.xforms, &(), &tables.brushes, &my_id)
        {
            for (xform, brush) in iter::zip(xforms, brushes) {
                let instance = SimpleCanvasInstance {
                        position: xform.xyz.into(),
                        rotation: xform.rot,
                    };
                self.sort.push((instance, brush.canvas));
            }
        }
    }

    fn pack(&mut self, adr: u32, draws: &mut Vec<CanvasDraw>) {
        self.instances.clear();
        if self.sort.is_empty() {
            return;
        }
        let mut run_start = 0;
        let mut run_canvas = self.sort[0].1;
        for (i, (inst, canvas)) in self.sort.iter().enumerate() {
            self.instances.push(*inst);
            if *canvas != run_canvas {
                draws.push(CanvasDraw {
                    canvas: run_canvas,
                    adr: adr + run_start as u32,
                    count: (i - run_start) as u32,
                });
                run_start = i;
                run_canvas = *canvas;
            }
        }
        let last = self.sort.len();
        draws.push(CanvasDraw {
            canvas: run_canvas,
            adr: adr + run_start as u32,
            count: (last - run_start) as u32,
        });
    }

    fn write(
        &self,
        encoder: &mut CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        instance_buffer: &Buffer,
        adr: u32,
    ) {
        if !self.instances.is_empty() {
            let instance_size = size_of::<SimpleCanvasInstance>() as u64;
            let byte_offset = (adr as u64) * instance_size;
            let bytes = self.instances.as_slice().as_bytes();
            let size = wgpu::BufferSize::new(bytes.len() as u64)
                .expect("instance buffer write size is non-zero");
            let mut view = belt.write_buffer(encoder, instance_buffer, byte_offset, size);
            view.copy_from_slice(bytes);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, zerocopy::IntoBytes, zerocopy::Immutable)]
struct SimpleCanvasInstance {
    position: Vec3A,//aligned vec
    rotation: Quat,
}

impl CanvasSolver for SimpleCanvasSolver {
    fn instance_size(&self) -> u32 {
        size_of::<SimpleCanvasInstance>() as u32
    }

    fn solve(
        &mut self,
        tables: &DemoTables,
        encoder: &mut CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        instance_buffer: &Buffer,
        solver_id: CanvasSolverId,
        adr: u32,
        draws: &mut Vec<CanvasDraw>,
    ) -> u32 {
        self.collect_from(tables, solver_id);
        self.sort.sort_by_key(|(_, c)| *c);
        self.pack(adr, draws);
        self.write(encoder, belt, instance_buffer, adr);
        self.instances.len() as u32
    }
}
