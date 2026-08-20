use core::range::Range;
use std::mem::size_of;

use zerocopy::IntoBytes as _;

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use glam::Vec2;
use slotmap::SecondaryMap;
use wgpu::util::{BufferInitDescriptor, DeviceExt, StagingBelt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferSize, BufferUsages, BufferViewMut, ColorTargetState, ColorWrites,
    CommandEncoder, CompareFunction, DepthStencilState, Device, FragmentState, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipelineDescriptor, Sampler,
    SamplerDescriptor, ShaderModuleDescriptor, ShaderStages, TextureFormat, TextureView,
    TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};

use crate::brushes::Brush;
use crate::gpuscope::canvasing::{CanvasSolver, CanvasTrait, CanvasUnderstander, DrawWriter, EveryCanvas};
use crate::gpuscope::texture_cache::{ImgId, TextureScope};
use crate::spacial::camera::Camera;
use crate::spacial::transform::Transform;
use crate::ecs::tables::Tables;
use crate::ecs::{CanvasId, MaterialId};

const MAX_MATERIALS: u64 = 128;

//todo, it should be allowed to get a texture view from elsewhere
pub struct SpriteMaterial {
    pub binding: BindGroup,
    pub billboard: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, zerocopy::IntoBytes, zerocopy::Immutable)]
struct SpriteUniformDatum {
    natural_scale: [f32; 3],
    y_offset: f32,
}

struct CameraUniforms {
    bind: BindGroup,
    buffer: Buffer,
    slice: Range<u64>,
}

struct QuadGeometry {
    buffer: Buffer,
    slice: Range<u64>,
}

struct MaterialUniforms {
    buffer: Buffer,
    stride: u64,
}

pub struct BasicSpriteCanvas {
    binds_layout: BindGroupLayout,

    materials: SecondaryMap<MaterialId, SpriteMaterial>,
    material_uniforms: MaterialUniforms,

    sampler: Sampler,
    pixels_per_unit: f32,

    camera: CameraUniforms,
    quad: QuadGeometry,
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
                        min_binding_size: BufferSize::new(size_of::<SpriteUniformDatum>() as u64),
                    },
                    count: None,
                },
            ],
        });

        let material_uniforms = make_material_uniforms(device);

        let shader = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/green_rect.wgsl"
        ));
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
                        array_stride: 20,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 12,
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
                                format: VertexFormat::Float16x4,
                                offset: 12,
                                shader_location: 3,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float16x4,
                                offset: 20,
                                shader_location: 4,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float16x4,
                                offset: 28,
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
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::LessEqual),
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

        let camera_slice = Range::from(0..64u64);

        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("sprite camera"),
            size: camera_slice.end,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sprite camera"),
            layout: &every.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &camera_buffer,
                    offset: camera_slice.start,
                    size: BufferSize::new(camera_slice.end - camera_slice.start),
                }),
            }],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("sprite sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let quad: [[f32; 5]; 6] = [
            [-0.5, -0.5, 0.0, 0.0, 1.0],
            [0.5, -0.5, 0.0, 1.0, 1.0],
            [0.5, 0.5, 0.0, 1.0, 0.0],
            [-0.5, 0.5, 0.0, 0.0, 0.0],
            [-0.5, -0.5, 0.0, 0.0, 1.0],
            [0.5, 0.5, 0.0, 1.0, 0.0],
        ];
        let quad_slice = Range::from(0..quad.as_bytes().len() as u64);

        let quad_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("quad"),
            contents: quad.as_bytes(),
            usage: BufferUsages::VERTEX,
        });

        (
            every,
            Self {
                binds_layout,
                materials: SecondaryMap::new(),
                material_uniforms,
                sampler,
                pixels_per_unit,
                camera: CameraUniforms {
                    bind: camera_bind,
                    buffer: camera_buffer,
                    slice: camera_slice,
                },
                quad: QuadGeometry {
                    buffer: quad_buffer,
                    slice: quad_slice,
                },
            },
        )
    }

    pub fn add_sprite(
        &mut self,
        device: &Device,
        queue: &Queue,
        every: &mut EveryCanvas,
        texture_scope: &TextureScope,
        img_id: ImgId,
        billboard: bool,
    ) -> Option<MaterialId> {
        if self.materials.len() as u64 >= MAX_MATERIALS {
            return None;
        }

        let texture = texture_scope.get(img_id)?;
        let (w, h) = (texture.width(), texture.height());
        let natural_scale = Vec2::new(w as f32, h as f32) / self.pixels_per_unit;

        let view = texture.create_view(&TextureViewDescriptor::default());

        let y_offset = if billboard { 0.5 } else { 0.0 };
        let uniform = SpriteUniformDatum {
            natural_scale: [natural_scale.x, natural_scale.y, 1.0],
            y_offset,
        };

        let material_index = self.materials.len() as u64;
        let offset = material_index * self.material_uniforms.stride;

        queue.write_buffer(&self.material_uniforms.buffer, offset, uniform.as_bytes());

        let binding = Self::create_material_binding(
            device,
            &self.binds_layout,
            &self.sampler,
            &self.material_uniforms.buffer,
            &view,
            offset,
        );

        let id = every.material_ids.insert(());
        self.materials.insert(id, SpriteMaterial { binding, billboard });
        Some(id)
    }

    fn create_material_binding(
        device: &Device,
        layout: &BindGroupLayout,
        sampler: &Sampler,
        buffer: &Buffer,
        view: &TextureView,
        offset: u64,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("sprite material"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer,
                        offset,
                        size: BufferSize::new(size_of::<SpriteUniformDatum>() as u64),
                    }),
                },
            ],
        })
    }
}

impl CanvasTrait for BasicSpriteCanvas {
    fn prepare(&mut self, camera: &Camera, encoder: &mut CommandEncoder, belt: &mut StagingBelt) {
        let array = camera.view_proj.to_cols_array();
        let bytes = array.as_bytes();
        let mut view = belt.write_buffer(
            encoder,
            &self.camera.buffer,
            self.camera.slice.start,
            BufferSize::new(self.camera.slice.end - self.camera.slice.start).unwrap(),
        );
        view.copy_from_slice(bytes);
    }

    fn begin_render_pass(&self, pass: &mut RenderPass<'_>, every: &EveryCanvas) {
        pass.set_pipeline(&every.pipeline);
        pass.set_bind_group(0, &self.camera.bind, &[]);
        pass.set_vertex_buffer(
            0,
            self.quad
                .buffer
                .slice(self.quad.slice.start..self.quad.slice.end),
        );
    }

    fn render(
        &self,
        pass: &mut RenderPass<'_>,
        material: MaterialId,
        instances: std::ops::Range<u32>,
        _every: &EveryCanvas,
    ) {
        let material = self.materials.get(material).expect("missing material");
        pass.set_bind_group(1, &material.binding, &[]);
        pass.draw(0..6, instances);
    }

}

pub struct SpriteCanvasSolver {
    bump: Bump,
    pub understanders: SecondaryMap<CanvasId, Box<dyn CanvasUnderstander<(Transform, Brush)>>>,
}

impl SpriteCanvasSolver {
    pub fn new() -> Self {
        Self {
            bump: Bump::with_capacity(1 << 20),
            understanders: SecondaryMap::new(),
        }
    }

    fn collect_sorted<'b>(
        tables: &Tables,
        bump: &'b Bump,
    ) -> BumpVec<'b, ((Transform, Brush), MaterialId, CanvasId)> {
        let mut sorted = BumpVec::new_in(bump);
        crate::query!(
            [&tables.core.xforms, &tables.core.brushes],
            |xform, brush| {
                sorted.push((
                    (xform.clone(), brush.clone()),
                    brush.material,
                    brush.canvas,
                ));
            }
        );
        sorted
    }

    fn pack_sorted(
        sorted: &[((Transform, Brush), MaterialId, CanvasId)],
        understanders: &mut SecondaryMap<CanvasId, Box<dyn CanvasUnderstander<(Transform, Brush)>>>,
        view: &mut BufferViewMut,
        byte_base: u64,
        adr: u32,
        sink: &mut DrawWriter,
    ) -> usize {
        let instance_size = size_of::<SpriteInstance>() as u64;
        let mut written: usize = 0;
        let mut canvas_start: usize = 0;
        let mut current_canvas: CanvasId = sorted[0].2;

        for i in 1..=sorted.len() {
            let end_of_canvas = i == sorted.len() || sorted[i].2 != current_canvas;
            if end_of_canvas {
                let slice = &sorted[canvas_start..i];
                let expected = slice.len() * instance_size as usize;
                let out = view.slice(
                    byte_base as usize + written..byte_base as usize + written + expected,
                );
                if let Some(canvas) = sink.get_canvas(current_canvas)
                    && let Some(understander) = understanders.get_mut(current_canvas)
                {
                    understander.understand(current_canvas, slice, out, canvas);
                }
                Self::pack_draws(
                    sink,
                    current_canvas,
                    slice,
                    adr + (written as u64 / instance_size) as u32,
                );
                written += expected;
                if i < sorted.len() {
                    canvas_start = i;
                    current_canvas = sorted[i].2;
                }
            }
        }

        written
    }

    fn pack_draws(
        sink: &mut DrawWriter,
        canvas_id: CanvasId,
        t: &[((Transform, Brush), MaterialId, CanvasId)],
        start: u32,
    ) {
        if t.is_empty() {
            return;
        }
        let mut run_start = 0;
        let mut run_material = t[0].1;
        for (i, (_, material, _)) in t.iter().enumerate() {
            if *material != run_material {
                sink.set_draw(
                    canvas_id,
                    run_material,
                    start + run_start as u32,
                    (i - run_start) as u32,
                );
                run_material = *material;
                run_start = i;
            }
        }
        let last = t.len();
        sink.set_draw(
            canvas_id,
            run_material,
            start + run_start as u32,
            (last - run_start) as u32,
        );
    }
}

impl CanvasSolver for SpriteCanvasSolver {
    fn solve(
        &mut self,
        tables: &Tables,
        view: &mut BufferViewMut,
        sink: &mut DrawWriter,
    ) -> usize {
        self.bump.reset();

        let mut sorted = Self::collect_sorted(tables, &self.bump);
        if sorted.is_empty() {
            return 0;
        }

        sorted
            .as_mut_slice()
            .sort_unstable_by_key(|(_, material, canvas)| (*canvas, *material));

        let instance_size = size_of::<SpriteInstance>() as u64;
        let (adr, byte_base) = sink.reserve(sorted.len() as u32, instance_size);

        Self::pack_sorted(
            &sorted,
            &mut self.understanders,
            view,
            byte_base,
            adr,
            sink,
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, zerocopy::IntoBytes, zerocopy::Immutable)]
pub(crate) struct SpriteInstance {
    pub(crate) position: [f32; 3],
    pub(crate) rotation: [f16; 4],
    pub(crate) scale: [f16; 4],
    pub(crate) color: [f16; 4],
}

fn make_material_uniforms(device: &Device) -> MaterialUniforms {
    let stride = (size_of::<SpriteUniformDatum>() as u64)
        .next_multiple_of(device.limits().min_uniform_buffer_offset_alignment as u64);

    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("sprite materials"),
        size: MAX_MATERIALS * stride,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    MaterialUniforms { buffer, stride }
}
