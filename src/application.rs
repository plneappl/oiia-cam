use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasRawDisplayHandle, HasWindowHandle};
use wgpu::naga::compact::KeepUnused::No;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::renderer::Renderer;

pub struct App<'a> {
    renderer: Option<Renderer<'a>>,
    size: crate::util::Size,
}

impl<'a> App<'a> {
    pub fn new(size: crate::util::Size) -> App<'a> {
        App {
            renderer: None,
            size: size,
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        window.set_visible(true);
        window.set_resizable(false);
        window.request_inner_size(Size::Physical(PhysicalSize::new(
            u32::try_from(self.size.w).unwrap(),
            u32::try_from(self.size.h).unwrap(),
        )));
        let handle = window.window_handle().unwrap();
        let renderer_fut = Renderer::new(event_loop.owned_display_handle(), window.clone());
        self.renderer = Some(pollster::block_on(renderer_fut));
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let renderer = self.renderer.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                renderer.render();
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }
            _ => (),
        }
    }
}
