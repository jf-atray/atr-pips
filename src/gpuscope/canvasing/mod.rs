use std::iter;
use std::mem::size_of;
use zerocopy::IntoBytes as _;

use glam::{Quat, Vec3A};
use slotmap::SlotMap;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, RenderPass,
};
use wgpu::util::DeviceExt as _;

use crate::query::impls::query_ref_ref;
use crate::spacial::camera::Camera;
use crate::tables::{CanvasId, CanvasSolverId};
use crate::tables::tables::Tables;

pub trait CanvasSolver {
    fn instance_size(&self) -> u32;

    fn solve(
        &mut self,
        tables: &Tables,
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

pub struct Canvas {
    pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
}

impl Canvas {
    pub fn new(
        device: &Device,
        format: wgpu::TextureFormat,
        sample_count: u32,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Self {
        const SHADER: &str = r#"
            @vertex
            fn vs_main(
                @location(0) local: vec2<f32>,
                @location(1) pos: vec3<f32>,
                @location(2) _rot: vec4<f32>,
            ) -> @builtin(position) vec4<f32> {
                return vec4<f32>(local * 0.1 + pos.xy, 0.0, 1.0);
            }

            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }
        "#;

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("green rect"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("green rect"),
            bind_group_layouts: &[],
            ..Default::default()
        });

        let quad: [[f32; 2]; 6] = [
            [-1.0, -1.0],
            [ 1.0, -1.0],
            [ 1.0,  1.0],
            [ 1.0,  1.0],
            [-1.0,  1.0],
            [-1.0, -1.0],
        ];
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad"),
            contents: quad.as_bytes(),
            usage: BufferUsages::VERTEX,
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("canvas camera"),
            size: 64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_stride = size_of::<SimpleCanvasInstance>() as u64;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("green rect"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main".into()),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: instance_stride,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 2,
                            },
                        ],
                    }),
                ],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: depth_format.map(|f| wgpu::DepthStencilState {
                format: f,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main".into()),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            quad_buffer,
            uniform_buffer,
        }
    }
}

pub struct CanvasRenderer {
    pub solvers: SlotMap<CanvasSolverId, Box<dyn CanvasSolver>>,
    pub canvases: SlotMap<CanvasId, Canvas>,

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
            canvases: SlotMap::with_key(),
            staging_belt,
            instance_buffer,
            draws: Vec::new(),
        }
    }

    pub fn prepare(
        &mut self,
        tables: &Tables,
        camera: &Camera,
        encoder: &mut CommandEncoder,
    ) -> &[CanvasDraw] {
        self.draws.clear();

        //gl vs glam style mat4s. todo, double check
        let array = camera.view_proj.to_cols_array();
        let bytes = array.as_bytes();
        for canvas in self.canvases.values() {
            //todo, each canvas may interpret the camera differently
            let mut view = self.staging_belt.write_buffer(
                encoder,
                &canvas.uniform_buffer,
                0,
                wgpu::BufferSize::new(64).unwrap(),
            );
            view.copy_from_slice(bytes);
        }

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

    pub fn render(&self, pass: &mut RenderPass<'_>) {
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        for draw in &self.draws {
            if let Some(canvas) = self.canvases.get(draw.canvas) {
                pass.set_pipeline(&canvas.pipeline);
                pass.set_vertex_buffer(0, canvas.quad_buffer.slice(..));
                pass.draw(0..6, draw.adr..draw.adr + draw.count);
            }
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

    fn collect_from(&mut self, tables: &Tables, _my_id: CanvasSolverId) {
        self.sort.clear();
        //our first real query!
        //next up, macro-assist this vec-of-vecs iteration
        for (xforms, brushes) in query_ref_ref(&tables.core.xforms, &(), &tables.core.brushes, &())
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
        tables: &Tables,
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
