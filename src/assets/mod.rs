use std::collections::HashMap;

use glam::Vec2;

use crate::tables::{CanvasId, MaterialId};

/// Sub-region within a sprite texture.
///
/// For whole textures this covers the full image; it can later describe a
/// sprite inside an atlas.
#[derive(Clone, Copy, Debug, Default)]
pub struct SpriteRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl SpriteRect {
    pub fn full(w: f32, h: f32) -> Self {
        Self { x: 0.0, y: 0.0, w, h }
    }

    pub fn uv_rect(&self, tex_w: f32, tex_h: f32) -> [f32; 4] {
        [
            self.x / tex_w,
            self.y / tex_h,
            (self.x + self.w) / tex_w,
            (self.y + self.h) / tex_h,
        ]
    }
}

/// A resolved sprite that the scene can instantiate by name.
#[derive(Clone, Copy, Debug)]
pub struct SpriteEntry {
    pub canvas: CanvasId,
    pub material: MaterialId,
    pub natural_scale: Vec2,
    pub rect: SpriteRect,
}

/// Maps logical sprite names to the canvas/material that render them.
///
/// The registry does not own GPU resources; it only stores handles into the
/// canvas material table and the derived natural scale for content authoring.
pub struct AssetRegistry {
    entries: HashMap<String, SpriteEntry>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, entry: SpriteEntry) {
        self.entries.insert(name.into(), entry);
    }

    pub fn get(&self, name: &str) -> &SpriteEntry {
        self.entries
            .get(name)
            .unwrap_or_else(|| panic!("AssetRegistry: sprite '{name}' not found"))
    }

    pub fn try_get(&self, name: &str) -> Option<&SpriteEntry> {
        self.entries.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(std::string::String::as_str)
    }

    pub fn pick_random<R: rand::Rng>(&self, rng: &mut R) -> &str {
        let idx = rng.random_range(0..self.entries.len());
        self.entries.keys().nth(idx).unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}
