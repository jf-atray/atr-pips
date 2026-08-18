use std::sync::Arc;
use std::time::{Duration, Instant};

use wgpu::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::WindowAttributes;

use crate::assets::AssetRegistry;
use crate::gamescope::game::Game;
use crate::you_first::gamejam::roller::overworld::OverworldScene;
use crate::gpuscope::{Gpu, GpuReady, GpuSettings};
use crate::libscope::Lib;
use crate::windowing::Windowing;

pub struct App {
    lib: Arc<Lib>,
    proxy: EventLoopProxy<GpuReady>,
    state: AppState,
    target_fps: Option<u32>,
    target_dt: Option<f32>,
    last_render: Option<Instant>,
    prev_tick: Option<Instant>,
    fps_frame_count: u32,
    fps_timer: f32,
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
            target_fps: None,
            target_dt: None,
            last_render: None,
            prev_tick: None,
            fps_frame_count: 0,
            fps_timer: 0.0,
        }
    }

    fn refresh_target_fps(&mut self) {
        let windowing = match &self.state {
            AppState::Windowed(w)
            | AppState::AwaitingGpu { windowing: w }
            | AppState::Ready { windowing: w, .. } => w,
            AppState::Boot => return,
        };

        self.target_dt = windowing.window.current_monitor().and_then(|monitor| {
            monitor
                .refresh_rate_millihertz()
                .map(|hz| hz as f32 / 1000.0)
        });
        self.target_fps = self
            .target_dt
            .map(|dt| dt.round().clamp(30.0, 240.0) as u32)
            .or(Some(60));
    }

    fn draw(&mut self) {
        let AppState::Ready {
            windowing,
            gpu,
            game,
        } = &mut self.state
        else {
            return;
        };

        let now = Instant::now();
        let elapsed = self
            .prev_tick
            .map_or(0.0, |prev| (now - prev).as_secs_f32().min(0.25));
        self.prev_tick = Some(now);

        let aspect = windowing.width as f32 / windowing.height as f32;
        game.update(elapsed, aspect, gpu);

        if let Some(mut frame) = gpu.begin_frame() {
            gpu.device.canvas_renderer.prepare(
                &game.domain.tables,
                &game.camera,
                &mut frame.encoder,
            );
            frame.with_render_pass(wgpu::Color::BLACK, |pass| {
                gpu.device.canvas_renderer.render(pass);
            });
            frame.finish(gpu.queue());
            gpu.device.canvas_renderer.recall();
        }

        self.fps_frame_count += 1;
        self.fps_timer += elapsed;
        if self.fps_timer >= 10.0 {
            let fps = self.fps_frame_count as f32 / self.fps_timer;
            log::info!("update rate: {fps:.2} Hz");
            self.fps_frame_count = 0;
            self.fps_timer = 0.0;
        }

        self.last_render = Some(Instant::now());
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
                self.refresh_target_fps();
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
                self.refresh_target_fps();
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

        self.refresh_target_fps();
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
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let AppState::Ready { game, .. } = &mut self.state {
                    if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        game.input.handle_event(crate::input::InputEvent::Key {
                            code,
                            state: event.state,
                            repeat: event.repeat,
                        });
                    }
                    if let Some(text) = event.text.as_ref() {
                        for c in text.chars() {
                            game.input.handle_event(crate::input::InputEvent::Char(c));
                        }
                    }
                }
            }
            WindowEvent::Ime(ime) => {
                if let AppState::Ready { game, .. } = &mut self.state
                    && let winit::event::Ime::Commit(text) = ime {
                        game.input.handle_event(crate::input::InputEvent::ImeCommit(text));
                    }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let AppState::Ready { game, .. } = &mut self.state {
                    game.input.handle_event(crate::input::InputEvent::MouseMove(glam::Vec2::new(
                        position.x as f32,
                        position.y as f32,
                    )));
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if let AppState::Ready { game, .. } = &mut self.state {
                    game.input.handle_event(crate::input::InputEvent::MouseButton { button, state });
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let AppState::Ready { game, .. } = &mut self.state {
                    let scroll = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => glam::Vec2::new(x, y),
                        winit::event::MouseScrollDelta::PixelDelta(p) => {
                            glam::Vec2::new(p.x as f32, p.y as f32) * 0.01
                        }
                    };
                    game.input.handle_event(crate::input::InputEvent::MouseScroll(scroll));
                }
            }
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

            const PIXELS_PER_UNIT: f32 = 512.0;

            let mut game = Game::new(AssetRegistry::new());
            game.set_scene(Box::new(OverworldScene::new(PIXELS_PER_UNIT)));

            self.state = AppState::Ready {
                windowing,
                gpu,
                game,
            };
            self.prev_tick = Some(Instant::now());
            self.last_render = Some(Instant::now());
        } else {
            log::warn!("received GpuReady in unexpected state");
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let windowing = if let AppState::Ready { windowing, .. } = &self.state {
            windowing
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
            return;
        };

        match self.target_fps {
            Some(fps) => {
                let interval = Duration::from_secs_f32(1.0 / fps as f32);
                let now = Instant::now();
                let last = self.last_render.unwrap_or(now);
                let next = if last + interval > now {
                    last + interval
                } else {
                    now + interval
                };
                event_loop.set_control_flow(ControlFlow::WaitUntil(next));
            }
            None => {
                event_loop.set_control_flow(ControlFlow::Poll);
            }
        }

        windowing.window.request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}
}
