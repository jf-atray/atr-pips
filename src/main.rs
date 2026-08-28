#![feature(inherent_associated_types)]
#![feature(iter_macro)]
#![feature(yield_expr)]
#![feature(f16)]
#![feature(trivial_bounds)]
#![feature(const_option_ops)]
#![feature(const_trait_impl)]
#![feature(stmt_expr_attributes)]


#![allow(incomplete_features)]
#![allow(dead_code)]
#![allow(clippy::cast_precision_loss, reason="Should levy f64 more.")]
mod addition;
mod anims;
mod appscope;
mod assets;
mod brushes;
mod canvases;
mod clip;
mod demo;
mod gamescope;
mod gpuscope;
mod input;
mod libscope;
mod physics;
mod query;
mod seek;
mod spacial;
mod ecs;
mod windowing;

pub use crate::ecs::gather;

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
