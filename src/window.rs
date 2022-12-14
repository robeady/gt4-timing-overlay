use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::glutin::{
    self,
    event::{Event, WindowEvent},
};
use glium::{Display, Surface};
use imgui::{Context, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

pub struct App {
    pub event_loop: EventLoop<()>,
    pub display: glium::Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub dpi_factor: f64,
}

impl App {
    pub fn init(title: &str, window_size: [f32; 2]) -> App {
        let event_loop = EventLoop::new();
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let builder = WindowBuilder::new()
            .with_title(title.to_owned())
            .with_inner_size(glutin::dpi::LogicalSize::new(window_size[0], window_size[1]));
        let display =
            Display::new(builder, context, &event_loop).expect("Failed to initialize display");

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
        }

        let dpi_factor = platform.hidpi_factor();

        let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        App { event_loop, display, imgui, platform, renderer, dpi_factor }
    }

    pub fn main_loop<F: FnMut(&mut Ui) + 'static>(
        self,
        mut run_ui: F,
        before_exit: impl Fn() + 'static,
    ) {
        let App { event_loop, display, mut imgui, mut platform, mut renderer, .. } = self;
        let mut last_frame = Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let mut ui = imgui.frame();
                run_ui(&mut ui);
                let gl_window = display.gl_window();
                let mut target = display.draw();
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                platform.prepare_render(&ui, gl_window.window());
                let draw_data = ui.render();
                renderer.render(&mut target, draw_data).expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
                before_exit();
            }
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
            }
        })
    }
}
