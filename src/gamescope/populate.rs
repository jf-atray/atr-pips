use std::collections::HashMap;
use std::fs;

use glam::Vec2;
use wgpu::{Device, Queue};

use crate::assets::SpriteEntry;
use crate::demo::canvasing::spritecanvas::BasicSpriteCanvas;
use crate::gamescope::game::Game;
use crate::gpuscope::canvasing::{CanvasRenderer, EveryCanvas};
use crate::gpuscope::texture_cache::TextureScope;
use crate::tables::{CanvasId, MaterialId};

pub fn build_game(
    device: &Device,
    queue: &Queue,
    canvas_renderer: &mut CanvasRenderer,
    texture_scope: &mut TextureScope,
    mut every: EveryCanvas,
    mut canvas: BasicSpriteCanvas,
    pixels_per_unit: f32,
    sprites_dir: &str,
) -> Game {
    let mut pending: Vec<(String, MaterialId, Vec2)> = Vec::new();

    {
        let name = "green";
        let white_pixel = texture_scope.white_pixel(device, queue);
        let material = canvas
            .add_sprite(device, queue, &mut every, texture_scope, white_pixel, false)
            .unwrap();
        pending.push((name.to_string(), material, Vec2::ONE));
    }

    if let Ok(entries) = fs::read_dir(sprites_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some(path_str) = path.to_str() else {
                continue;
            };
            let Some(img_id) = texture_scope.load_image(device, queue, path_str)
            else {
                log::warn!("failed to load sprite {}", path.display());
                continue;
            };
            let material = canvas
                .add_sprite(device, queue, &mut every, texture_scope, img_id, false)
                .unwrap();
            let (w, h) = texture_scope.size(img_id).unwrap();
            let natural_scale = Vec2::new(w as f32, h as f32) / pixels_per_unit;
            pending.push((name.to_string(), material, natural_scale));
        }
    }

    let canvas_id: CanvasId = canvas_renderer.canvases.insert((every, Box::new(canvas)));

    let mut registry = HashMap::new();
    for (name, material, natural_scale) in pending {
        registry.insert(
            name,
            SpriteEntry {
                canvas: canvas_id,
                material,
                natural_scale,
            },
        );
    }

    Game::new(registry)
}
