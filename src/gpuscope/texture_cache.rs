use std::collections::HashMap;

use image::ImageReader;
use slotmap::SlotMap;
use wgpu::{
    Device, Extent3d, Queue, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, util::DeviceExt, wgt::TextureDataOrder,
};

slotmap::new_key_type! {
    pub struct ImgId;
}

#[derive(Debug)]
pub struct TextureScope {
    names: HashMap<String, ImgId>,
    textures: SlotMap<ImgId, Texture>,
}

impl TextureScope {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            textures: SlotMap::with_key(),
        }
    }

    pub fn load_image(&mut self, device: &Device, queue: &Queue, path: &str) -> Option<ImgId> {
        if let Some(id) = self.names.get(path) {
            return Some(*id);
        }

        let image = ImageReader::open(path).ok()?.decode().ok()?;
        let rgba = image.to_rgba8();
        let (width, height) = (image.width(), image.height());

        Some(self.upload(device, queue, path, rgba.as_raw(), width, height))
    }

    pub fn load_from_bytes(
        &mut self,
        device: &Device,
        queue: &Queue,
        name: &str,
        data: &[u8],
    ) -> Option<ImgId> {
        if let Some(id) = self.names.get(name) {
            return Some(*id);
        }

        let image = image::load_from_memory(data).ok()?;
        let rgba = image.to_rgba8();
        let (width, height) = (image.width(), image.height());

        Some(self.upload(device, queue, name, rgba.as_raw(), width, height))
    }

    //oh god no
    pub fn white_pixel(&mut self, device: &Device, queue: &Queue) -> ImgId {
        if let Some(id) = self.names.get("__white_pixel") {
            return *id;
        }

        self.upload(
            device,
            queue,
            "__white_pixel",
            &[255u8, 255, 255, 255],
            1,
            1,
        )
    }

    pub fn get(&self, id: ImgId) -> Option<&Texture> {
        self.textures.get(id)
    }

    pub fn id(&self, name: &str) -> Option<ImgId> {
        self.names.get(name).copied()
    }

    pub fn size(&self, id: ImgId) -> Option<(u32, u32)> {
        self.textures.get(id).map(|t| (t.width(), t.height()))
    }

    fn upload(
        &mut self,
        device: &Device,
        queue: &Queue,
        name: &str,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> ImgId {
        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some(name),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            TextureDataOrder::LayerMajor,
            rgba,
        );

        let id = self.textures.insert(texture);
        self.names.insert(name.to_string(), id);
        id
    }
}
