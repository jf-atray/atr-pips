use std::cell::RefCell;
use std::fs;

use glam::Vec2;
use wgpu::{Device, Queue};

use crate::assets::{AssetRegistry, SpriteEntry, SpriteRect};
use crate::gamescope::game::Game;
use crate::gamescope::green_rect::SpriteCanvas;
use crate::gamescope::scene::act::{EveryScript, MyScript, OtherScript, ScriptHost};
use crate::gamescope::scene::{Scene, SceneAccess};
use crate::gpuscope::canvasing::{CanvasRenderer, EveryCanvas};
use crate::gpuscope::texture_cache::TextureScope;
use crate::tables::{CanvasId, MaterialId};

/// Build the Game, `AssetRegistry`, Scene, and all sprite materials from a `SpriteCanvas`
/// that has already been constructed in App.
pub fn build_game(
    device: &Device,
    queue: &Queue,
    canvas_renderer: &mut CanvasRenderer,
    texture_scope: &mut TextureScope,
    mut every: EveryCanvas,
    mut canvas: SpriteCanvas,
    pixels_per_unit: f32,
    sprites_dir: &str,
) -> Game {
    let mut pending: Vec<(String, MaterialId, Vec2, SpriteRect)> = Vec::new();

    for (name, color) in [
        ("green", [0.0f32, 1.0, 0.0, 1.0]),
        ("red", [1.0f32, 0.0, 0.0, 1.0]),
        ("blue", [0.0f32, 0.0, 1.0, 1.0]),
        ("yellow", [1.0f32, 1.0, 0.0, 1.0]),
    ] {
        let material = canvas.add_material(device, queue, &mut every, texture_scope, color);
        pending.push((
            name.to_string(),
            material,
            Vec2::ONE,
            SpriteRect::full(1.0, 1.0),
        ));
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
            let Some(img_id) = texture_scope.load_image(device, queue, path.to_str().unwrap_or(""))
            else {
                log::warn!("failed to load sprite {}", path.display());
                continue;
            };
            let material = canvas.add_sprite(device, queue, &mut every, texture_scope, img_id, [1.0f32; 4]);
            let (w, h) = texture_scope.size(img_id).unwrap();
            let natural_scale = Vec2::new(w as f32, h as f32) / pixels_per_unit;
            pending.push((
                name.to_string(),
                material,
                natural_scale,
                SpriteRect::full(w as f32, h as f32),
            ));
        }
    }

    let canvas_id: CanvasId = canvas_renderer.canvases.insert((every, Box::new(canvas)));

    let mut registry = AssetRegistry::new();
    for (name, material, natural_scale, rect) in pending {
        registry.register(
            name,
            SpriteEntry {
                canvas: canvas_id,
                material,
                natural_scale,
                rect,
            },
        );
    }

    let scene = Scene::demo(&registry, pixels_per_unit);
    let mut game = Game::new(SceneAccess { current: scene }, registry);
    game.load();

    let id_2 = game.scripts.scripts.insert(RefCell::new(ScriptHost::new(
        EveryScript { enabled: true },
        Box::new(OtherScript { num: 0 }),
    )));
    game.scripts.scripts.insert(RefCell::new(ScriptHost::new(
        EveryScript { enabled: true },
        Box::new(MyScript {
            player: None,
            other_script: Some(id_2),
        }),
    )));

    game
}
