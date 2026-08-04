use std::rc::Rc;

use wgpu::{
    Color, CommandEncoder, LoadOp, Operations, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, SurfaceTexture, Texture,
    TextureViewDescriptor,
};

use super::targets::RenderTargets;

pub struct Frame {
    pub(crate) surface_texture: SurfaceTexture,
    pub(crate) encoder: CommandEncoder,
    pub(crate) msaa_color: Option<Rc<Texture>>,
    pub(crate) depth: Option<Rc<Texture>>,
}

impl Frame {
    pub(crate) fn new(
        surface_texture: SurfaceTexture,
        encoder: CommandEncoder,
        targets: &RenderTargets,
    ) -> Self {
        Self {
            surface_texture,
            encoder,
            msaa_color: targets.msaa_color.clone(),
            depth: targets.depth.clone(),
        }
    }

    //scoped lifetime renderpass
    pub fn with_render_pass<'f, R>(
        &'f mut self,
        clear_color: Color,
        f: impl FnOnce(&mut RenderPass<'f>) -> R,
    ) -> R {
        let surface_view = self.surface_texture.texture.create_view(&TextureViewDescriptor::default());

        let msaa_view = self
            .msaa_color
            .as_ref()
            .map(|m| m.create_view(&TextureViewDescriptor::default()));

        let (color_view, resolve_target) = match &msaa_view {
            Some(v) => (v, Some(&surface_view)),
            None => (&surface_view, None),
        };

        let depth_view = self
            .depth
            .as_ref()
            .map(|d| d.create_view(&TextureViewDescriptor::default()));

        let depth_stencil = depth_view.as_ref().map(|view| RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            stencil_ops: None,
        });

        let mut pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("frame"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: color_view,
                depth_slice: None,
                resolve_target,
                ops: Operations {
                    load: LoadOp::Clear(clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: depth_stencil,
            ..Default::default()
        });

        f(&mut pass)
    }

    pub fn finish(self, queue: &Queue) {
        queue.submit([self.encoder.finish()]);
        queue.present(self.surface_texture);
    }
}
