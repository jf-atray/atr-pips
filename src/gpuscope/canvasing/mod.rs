use zerocopy::IntoBytes as _;

use slotmap::SlotMap;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferBindingType, BufferDescriptor, BufferSize, BufferUsages,
    CommandEncoder, Device, RenderPass, ShaderStages,
};
use wgpu::util::DeviceExt as _;

use crate::spacial::camera::Camera;
use crate::tables::{CanvasId, CanvasSolverId};
use crate::tables::tables::Tables;

pub trait CanvasSolver {
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

pub struct EveryCanvas {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub quad_buffer: Buffer,
}

impl EveryCanvas {
    pub fn new(
        device: &Device,
        format: wgpu::TextureFormat,
        sample_count: u32,
        depth_format: Option<wgpu::TextureFormat>,
        shader_source: &str,
        instance_stride: u64,
    ) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("canvas shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("canvas camera"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(64),
                },
                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("canvas"),
            bind_group_layouts: &[Some(&bind_group_layout)],
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

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("canvas"),
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
            bind_group_layout,
            pipeline,
            quad_buffer,
        }
    }
}

pub trait CanvasTrait {
    fn prepare(&mut self, camera: &Camera, encoder: &mut CommandEncoder, belt: &mut wgpu::util::StagingBelt);
    fn render(&self, pass: &mut RenderPass<'_>, instances: std::ops::Range<u32>, every: &EveryCanvas);
}


pub struct CanvasRenderer {
    pub solvers: SlotMap<CanvasSolverId, Box<dyn CanvasSolver>>,
    pub canvases: SlotMap<CanvasId, (EveryCanvas, Box<dyn CanvasTrait>)>,

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

        for (_every, canvas) in self.canvases.values_mut() {
            canvas.prepare(camera, encoder, &mut self.staging_belt);
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
            if let Some((every, canvas)) = self.canvases.get(draw.canvas) {
                canvas.render(pass, draw.adr..draw.adr + draw.count, every);
            }
        }
    }

    pub fn recall(&mut self) {
        self.staging_belt.recall();
    }
}

