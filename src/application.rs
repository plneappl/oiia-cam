use std::sync::atomic::AtomicPtr;
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel};
use std::sync::{Arc, Mutex};

use raw_window_handle::{HasDisplayHandle, HasRawDisplayHandle, HasWindowHandle};
use wgpu::naga::compact::KeepUnused::No;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::object::Object;
use crate::renderer::Renderer;
use crate::resources::Resources;
use crate::state::State;
use crate::texture::Textures;
use crate::util::Vec2d;

pub struct App<'a> {
    pub renderer: Option<Renderer<'a>>,
    size: crate::util::Size,
    resources: &'a Resources,
    textures: Option<Textures<'a>>,
    state: Arc<Mutex<State<'a>>>,
    sender: SyncSender<Vec<u8>>,
}

impl<'a> App<'a> {
    pub fn new(
        size: crate::util::Size,
        resources: &'a Resources,
        state: Arc<Mutex<State<'a>>>,
        sender: SyncSender<Vec<u8>>,
    ) -> App<'a> {
        App {
            renderer: None,
            size: size,
            resources: resources,
            textures: None,
            sender: sender,
            state: state,
        }
    }

    pub fn renderer(&self) -> &Renderer<'a> {
        self.renderer.as_ref().unwrap()
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
        let renderer_fut = Renderer::new(
            event_loop.owned_display_handle(),
            window.clone(),
            self.resources,
            self.sender.clone(),
        );
        let (renderer, textures) = pollster::block_on(renderer_fut);
        self.renderer = Some(renderer);
        self.textures = Some(textures);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let renderer = self.renderer.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");

                renderer.exit();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let textures = self.textures.as_mut().unwrap();
                let objs = self.state.lock().unwrap().render_gpu(&textures);
                renderer.render(objs);
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }
            _ => (),
        }
    }
}
