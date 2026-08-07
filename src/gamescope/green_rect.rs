use std::iter;
use std::mem::size_of;
use zerocopy::IntoBytes as _;

use glam::{Quat, Vec3A};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferSize,
    BufferUsages, CommandEncoder, Device, RenderPass, TextureFormat,
};
use wgpu::util::StagingBelt;

use crate::brushes::Brush;
use crate::gpuscope::canvasing::{CanvasDraw, CanvasSolver, CanvasTrait, EveryCanvas};
use crate::query::impls::query_ref_ref;
use crate::spacial::camera::Camera;
use crate::spacial::transform::Transform;
use crate::tables::{CanvasId, CanvasSolverId};
use crate::tables::tables::Tables;

pub struct GreenRectCanvas {
    uniform_buffer: Buffer,
    bind_group: BindGroup,
}

impl GreenRectCanvas {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        sample_count: u32,
        depth_format: Option<TextureFormat>,
    ) -> (EveryCanvas, Box<dyn CanvasTrait>) {
        let shader = include_str!("green_rect.wgsl");
        let instance_stride = size_of::<GreenRectInstance>() as u64;
        let every = EveryCanvas::new(device, format, sample_count, depth_format, shader, instance_stride);

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("green rect camera"),
            size: 64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("green rect camera"),
            layout: &every.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let canvas = Self {
            uniform_buffer,
            bind_group,
        };
        let canvas: Box<dyn CanvasTrait> = Box::new(canvas);
        (every, canvas)
    }
}

impl CanvasTrait for GreenRectCanvas {
    fn prepare(&mut self, camera: &Camera, encoder: &mut CommandEncoder, belt: &mut StagingBelt) {
        let array = camera.view_proj.to_cols_array();
        let bytes = array.as_bytes();
        let mut view = belt.write_buffer(
            encoder,
            &self.uniform_buffer,
            0,
            BufferSize::new(64).unwrap(),
        );
        view.copy_from_slice(bytes);
    }

    fn render(&self, pass: &mut RenderPass<'_>, instances: std::ops::Range<u32>, every: &EveryCanvas) {
        pass.set_pipeline(&every.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, every.quad_buffer.slice(..));
        pass.draw(0..6, instances);
    }
}

pub struct GreenRectSolver {
    sort: Vec<(GreenRectInstance, CanvasId)>,
    instances: Vec<GreenRectInstance>,
}

impl GreenRectSolver {
    pub fn new() -> Self {
        Self {
            sort: Vec::new(),
            instances: Vec::new(),
        }
    }

    fn collect_from(&mut self, tables: &Tables, _my_id: CanvasSolverId) {
        self.sort.clear();
        for (xforms, brushes) in query_ref_ref(&tables.core.xforms, &(), &tables.core.brushes, &())
        {
            for (xform, brush) in iter::zip(xforms, brushes) {
                let instance = GreenRectInstance {
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
        belt: &mut StagingBelt,
        instance_buffer: &Buffer,
        adr: u32,
    ) {
        if !self.instances.is_empty() {
            let instance_size = size_of::<GreenRectInstance>() as u64;
            let byte_offset = (adr as u64) * instance_size;
            let bytes = self.instances.as_slice().as_bytes();
            let size = BufferSize::new(bytes.len() as u64)
                .expect("instance buffer write size is non-zero");
            let mut view = belt.write_buffer(encoder, instance_buffer, byte_offset, size);
            view.copy_from_slice(bytes);
        }
    }
}

impl CanvasSolver for GreenRectSolver {
    fn solve(
        &mut self,
        tables: &Tables,
        encoder: &mut CommandEncoder,
        belt: &mut StagingBelt,
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, zerocopy::IntoBytes, zerocopy::Immutable)]
struct GreenRectInstance {
    position: Vec3A,
    rotation: Quat,
}
