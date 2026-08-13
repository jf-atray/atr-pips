#![feature(inherent_associated_types)]
#![feature(f16)]
#![feature(trivial_bounds)]
#![allow(incomplete_features)]
#![allow(dead_code)]
mod anims;
mod appscope;
mod assets;
mod brushes;
mod canvases;
mod clip;
mod gamescope;
mod gather;
mod gpuscope;
mod libscope;
mod physics;
mod query;
mod demo;
mod scripting;
mod seek;
mod spacial;
mod tables;
mod windowing;

use std::sync::Arc;
use winit::event_loop;

#[cfg(not(target_arch = "wasm32"))]
fn main() {

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let event_loop = event_loop::EventLoop::<gpuscope::GpuReady>::with_user_event()
        .build()
        .expect("failed to create event loop");

    let proxy = event_loop.create_proxy();
    let lib = Arc::new(libscope::Lib::new(event_loop.owned_display_handle()));
    let mut app = appscope::App::new(lib, proxy);

    event_loop.run_app(&mut app).expect("event loop failed");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    unimplemented!("wip dude");
}
