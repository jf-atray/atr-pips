use std::iter;
use std::mem::size_of;
use core::range::Range;
use zerocopy::IntoBytes as _;

use glam::{Quat, Vec2, Vec3A, Vec4};
use slotmap::SecondaryMap;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferSize,
    BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthStencilState,
    Device, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPass,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModuleDescriptor, ShaderStages,
    BindingResource, Queue, TextureFormat, TextureViewDescriptor,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use wgpu::util::{BufferInitDescriptor, DeviceExt, StagingBelt};

use crate::gpuscope::canvasing::{CanvasSolver, CanvasTrait, DrawWriter, EveryCanvas};
use crate::gpuscope::texture_cache::{ImgId, TextureScope};
use crate::query::impls::query_ref_ref;
use crate::spacial::camera::Camera;
use crate::tables::{CanvasId, CanvasSolverId, MaterialId};
use crate::tables::tables::Tables;

//todo, it's actually decently logical to have a lifetime scope
//IE, this simply declares a slice('a) and 'a is gifted by a suballocator
//at the gpu-scope level
pub struct SpriteMaterial {
    pub binding: BindGroup,

    pub uniform_buffer: Buffer,
    pub uniform_buffer_slice: Range<u64>,
    
    pub natural_scale: Vec2,
}


#[repr(C)]
#[derive(Clone, Debug, zerocopy::IntoBytes, zerocopy::Immutable)]
struct SpriteUniformDatum {
    natural_scale: Vec2,
}



pub struct BasicSpriteCanvas {
    binds_layout: BindGroupLayout,

    materials: SecondaryMap<MaterialId, SpriteMaterial>,

    sampler: Sampler,
    pixels_per_unit: f32,

    camera_bind: BindGroup,
    camera_buffer: Buffer,
    camera_buffer_slice: core::range::Range<u64>,

    quad_buffer: Buffer,
    quad_buffer_slice: core::range::Range<u64>,
}

impl BasicSpriteCanvas {
    pub fn make(
        device: &Device,
        format: TextureFormat,
        sample_count: u32,
        depth_format: Option<TextureFormat>,
        pixels_per_unit: f32,
    ) -> (EveryCanvas, Self) {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sprite camera"),
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

        let binds_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sprite material"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(32),
                    },
                    count: None,
                },
            ],
        });

        let shader = include_str!( concat!(env!("CARGO_MANIFEST_DIR"),"/assets/shaders/green_rect.wgsl"));
        let instance_stride = size_of::<SpriteInstance>() as u64;

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("sprite"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&binds_layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("sprite"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                buffers: &[
                    Some(VertexBufferLayout {
                        array_stride: 16,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    }),
                    Some(VertexBufferLayout {
                        array_stride: instance_stride,
                        step_mode: VertexStepMode::Instance,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 2,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 3,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 32,
                                shader_location: 4,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                offset: 48,
                                shader_location: 5,
                            },
                        ],
                    }),
                ],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState::default(),
            depth_stencil: depth_format.map(|f| DepthStencilState {
                format: f,
                depth_write_enabled: Some(false),
                depth_compare: Some(CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            fragment: Some(FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        let every = EveryCanvas::new(bind_group_layout, pipeline);

        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("sprite camera"),
            size: 64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sprite camera"),
            layout: &every.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("sprite sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let quad: [[f32; 4]; 6] = [
            [-1.0, -1.0, 0.0, 1.0],
            [ 1.0, -1.0, 1.0, 1.0],
            [ 1.0,  1.0, 1.0, 0.0],

            [-1.0,  1.0, 0.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0],
            [ 1.0,  1.0, 1.0, 0.0],
        ];
        let quad_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("quad"),
            contents: quad.as_bytes(),
            usage: BufferUsages::VERTEX,
        });

        (every, Self {
            camera_buffer,
            camera_bind,
            camera_buffer_slice: Range::from(0..64u64),
            binds_layout,
            materials: SecondaryMap::new(),
            quad_buffer,
            quad_buffer_slice: Range::from(0..quad.as_bytes().len() as u64),
            sampler,
            pixels_per_unit,
        })
    }

    pub fn add_sprite(
        &mut self,
        device: &Device, //todo, should be allowed to get buffer from elsewhere
        //OR, is there advantage to laying them out in 1 buffer claimed by canvas?
        _queue: &Queue,
        every: &mut EveryCanvas,
        texture_scope: &TextureScope, //why ask the canvas to do the GET. Why not do the GET yourself?
        img_id: ImgId,
    ) -> Option<MaterialId> {
        let texture = texture_scope.get(img_id) ?;
        let (w, h) = (texture.width(), texture.height());
        let natural_scale = Vec2::new(w as f32, h as f32) / self.pixels_per_unit;

        let view = texture.create_view(&TextureViewDescriptor::default());

        let uniform = SpriteUniformDatum {
            natural_scale,
        };
        //ew ew etc
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sprite material"),
            contents: uniform.as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sprite material"),
            layout: &self.binds_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let id = every.material_ids.insert(());
        self.materials.insert(id, SpriteMaterial {
            uniform_buffer,
            uniform_buffer_slice: Range::from(0..uniform.as_bytes().len() as u64),
            binding: bind_group,
            natural_scale,
        });
        Some(id)
    }

}

impl CanvasTrait for BasicSpriteCanvas {
    fn prepare(&mut self, camera: &Camera, encoder: &mut CommandEncoder, belt: &mut StagingBelt) {
        let array = camera.view_proj.to_cols_array();
        let bytes = array.as_bytes();
        let mut view = belt.write_buffer(
            encoder,
            &self.camera_buffer,
            0,
            BufferSize::new(64).unwrap(),
        );
        view.copy_from_slice(bytes);
    }

    fn begin_render_pass(&self, pass: &mut RenderPass<'_>, every: &EveryCanvas) {
        pass.set_pipeline(&every.pipeline);
        pass.set_bind_group(0, &self.camera_bind, &[]);
        pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
    }

    fn render(&self, pass: &mut RenderPass<'_>, material: MaterialId, instances: std::ops::Range<u32>, _every: &EveryCanvas) {
        let material = self.materials.get(material).expect("missing material");
        pass.set_bind_group(1, &material.binding, &[]);
        pass.draw(0..6, instances);
    }
}

pub struct SpriteSolver {
    sort: Vec<(SpriteInstance, CanvasId, MaterialId)>,
    instances: Vec<SpriteInstance>,
}

impl SpriteSolver {
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
                let instance = SpriteInstance {
                    position: xform.xyz.into(),
                    rotation: xform.rot,
                    scale: brush.scale,
                    _pad: [0.0; 2],
                    color: brush.color,
                };
                self.sort.push((instance, brush.canvas, brush.material));
            }
        }
    }

    fn pack(&mut self, adr: u32, sink: &mut DrawWriter) {
        self.instances.clear();
        if self.sort.is_empty() {
            return;
        }
        let mut run_start = 0;
        let mut run_canvas = self.sort[0].1;
        let mut run_material = self.sort[0].2;
        for (i, (inst, canvas, material)) in self.sort.iter().enumerate() {
            self.instances.push(*inst);
            if *canvas != run_canvas || *material != run_material {
                sink.set_draw(run_canvas, run_material, adr + run_start as u32, (i - run_start) as u32);
                run_start = i;
                run_canvas = *canvas;
                run_material = *material;
            }
        }
        let last = self.sort.len();
        sink.set_draw(run_canvas, run_material, adr + run_start as u32, (last - run_start) as u32);
    }

    fn write(
        &self,
        encoder: &mut CommandEncoder,
        belt: &mut StagingBelt,
        instance_buffer: &Buffer,
        adr: u32,
    ) {
        if !self.instances.is_empty() {
            let instance_size = size_of::<SpriteInstance>() as u64;
            let byte_offset = u64::from(adr) * instance_size;
            let bytes = self.instances.as_slice().as_bytes();
            let size = BufferSize::new(bytes.len() as u64)
                .expect("instance buffer write size is non-zero");
            let mut view = belt.write_buffer(encoder, instance_buffer, byte_offset, size);
            view.copy_from_slice(bytes);
        }
    }
}

impl CanvasSolver for SpriteSolver {
    fn solve(
        &mut self,
        tables: &Tables,
        encoder: &mut CommandEncoder,
        belt: &mut StagingBelt,
        instance_buffer: &Buffer,
        solver_id: CanvasSolverId,
        sink: &mut DrawWriter,
    ) {
        self.collect_from(tables, solver_id);
        self.sort.sort_by_key(|(_, c, m)| (*c, *m));
        let total = self.sort.len() as u32;
        let adr = sink.reserve(total);
        self.pack(adr, sink);
        self.write(encoder, belt, instance_buffer, adr);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, zerocopy::IntoBytes, zerocopy::Immutable)]
struct SpriteInstance {
    position: Vec3A,
    rotation: Quat,
    scale: Vec2,
    _pad: [f32; 2],
    color: Vec4,
}
