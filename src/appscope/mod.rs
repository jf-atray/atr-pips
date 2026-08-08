use std::sync::Arc;

use wgpu::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::WindowAttributes;

use crate::gamescope::game::Game;
use crate::gamescope::green_rect::{GreenRectCanvas, GreenRectSolver};
use crate::gpuscope::{Gpu, GpuReady, GpuSettings};
use crate::libscope::Lib;
use crate::windowing::Windowing;

pub struct App {
    lib: Arc<Lib>,
    proxy: EventLoopProxy<GpuReady>,
    state: AppState,
}

pub enum AppState {
    Boot,
    Windowed(Windowing),
    AwaitingGpu {
        windowing: Windowing,
    },
    Ready {
        windowing: Windowing,
        gpu: Gpu,
        game: Game,
    },
}

impl App {
    pub fn new(lib: Arc<Lib>, proxy: EventLoopProxy<GpuReady>) -> Self {
        Self {
            lib,
            proxy,
            state: AppState::Boot,
        }
    }

    fn spawn_gpu_init(&self, windowing: &Windowing, surface: Surface<'static>) {
        let lib = self.lib.clone();
        let (w, h) = (windowing.width, windowing.height);
        let proxy = self.proxy.clone();

        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let parts = pollster::block_on(lib.make_gpu_parts(surface, w, h));
            let _ = proxy.send_event(GpuReady(parts));
        });

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (lib, surface, w, h, proxy);
            unimplemented!("browsers i na bit buddy");
        }
    }

    fn handle_resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        match &mut self.state {
            AppState::AwaitingGpu { windowing } => {
                windowing.set_size(width, height);
            }
            AppState::Ready { windowing, gpu, .. } => {
                windowing.set_size(width, height);
                gpu.reconfigure(width, height);
            }
            AppState::Windowed(_) => {
                //just knick these for a minute
                let AppState::Windowed(mut windowing) =
                    std::mem::replace(&mut self.state, AppState::Boot)
                else {
                    unreachable!()
                };
                let surface = self.lib.make_surface(windowing.window.clone());
                windowing.set_size(width, height);
                self.spawn_gpu_init(&windowing, surface);
                self.state = AppState::AwaitingGpu { windowing };
            }
            _ => {}
        }
    }
}

impl ApplicationHandler<GpuReady> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title("atr-pips")
            .with_inner_size(LogicalSize::new(1280.0, 720.0));

        let window = event_loop
            .create_window(attributes)
            .expect("failed to create window");

        let inner = window.inner_size();
        let windowing = Windowing::new(window, inner.width, inner.height);

        if windowing.is_zero() {
            log::info!("window is 0x0; waiting for resize before GPU init");
            self.state = AppState::Windowed(windowing);
        } else {
            let surface = self.lib.make_surface(windowing.window.clone());
            self.spawn_gpu_init(&windowing, surface);
            self.state = AppState::AwaitingGpu { windowing };
        }

        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.handle_resize(size.width, size.height),
            _ => {}
        }
    }

    //submission of window resources
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: GpuReady) {
        if let AppState::AwaitingGpu { windowing } =
            std::mem::replace(&mut self.state, AppState::Boot)
        {
            let mut gpu = Gpu::make(event.0, &GpuSettings::default());
            gpu.reconfigure(windowing.width, windowing.height);


            let (every, canvas, default_material) = GreenRectCanvas::new(
                &gpu.device.device,
                gpu.surface.cfg.format,
                gpu.targets.sample_count(),
                gpu.targets
                    .depth_enabled()
                    .then(|| wgpu::TextureFormat::Depth32Float),
            );
            let canvas_id = gpu.device.canvas_renderer.canvases.insert((every, canvas));
            let _solver_id = gpu
                .device
                .canvas_renderer
                .solvers
                .insert(Box::new(GreenRectSolver::new()));
            
            //todo, gamedata invariant should not be tied to the device driver.
            let mut game = Game::new();
            game.populate(canvas_id, default_material);
            self.state = AppState::Ready {
                windowing,
                gpu,
                game,
            };
        } else {
            log::warn!("received GpuReady in unexpected state");
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let AppState::Ready {
            windowing,
            gpu,
            game,
        } = &mut self.state
        {
            let aspect = windowing.width as f32 / windowing.height as f32;
            game.update(0.016, aspect);

            if let Some(mut frame) = gpu.begin_frame() {
                gpu.device
                    .canvas_renderer //create some transient thin scope structs
                    .prepare(&game.domain.tables, &game.camera, &mut frame.encoder);
                frame.with_render_pass(wgpu::Color::BLACK, |pass| {
                    gpu.device.canvas_renderer.render(pass);
                });
                frame.finish(gpu.queue());
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}
}
