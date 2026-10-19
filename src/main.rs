mod desktop;
mod render;

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use render::Renderer;

const DEFAULT_SHADER: &str = include_str!("../shaders/default.wgsl");
const FRAME_TIME: Duration = Duration::from_millis(1000 / 60);

struct App {
    shader_src: String,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    next_frame: Instant,
}

impl App {
    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let attrs = Window::default_attributes()
            .with_title("shitass-wallpaper")
            .with_decorations(false)
            .with_visible(false);
        let window = Arc::new(event_loop.create_window(attrs)?);

        let RawWindowHandle::Win32(handle) = window.window_handle()?.as_raw() else {
            anyhow::bail!("not a Win32 window");
        };
        // Reparent before creating the surface so the swapchain binds to the final window.
        desktop::attach_to_desktop(HWND(handle.hwnd.get() as *mut _))?;

        self.renderer = Some(Renderer::new(window.clone(), &self.shader_src)?);
        self.window = Some(window);
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            if let Err(e) = self.init(event_loop) {
                eprintln!("init failed: {e:#}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                // Child windows get no mouse input, so poll the global cursor instead.
                let mouse = desktop::cursor_pos()
                    .map(|(x, y)| (x as f32, y as f32))
                    .unwrap_or((0.0, 0.0));
                if let Some(r) = &mut self.renderer {
                    r.render(mouse);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if now >= self.next_frame {
            self.next_frame = now + FRAME_TIME;
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
    }
}

fn main() -> Result<()> {
    desktop::set_dpi_aware();

    // Optional: `shitass-wallpaper path/to/shader.wgsl`
    let shader_src = match std::env::args().nth(1) {
        Some(path) => std::fs::read_to_string(path)?,
        None => DEFAULT_SHADER.to_string(),
    };

    let event_loop = EventLoop::new()?;
    let mut app = App {
        shader_src,
        window: None,
        renderer: None,
        next_frame: Instant::now(),
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
