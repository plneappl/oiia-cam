use std::default;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use raw_window_handle::{HasDisplayHandle, HasRawDisplayHandle, HasWindowHandle};
use virtualcam::{Camera, PixelFormat};
use wgpu::naga::compact::KeepUnused::No;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::animation::{self, Animation};
use crate::bounce::Bounce;
use crate::imageanim::ImageAnim;
use crate::microphone;
use crate::object::Object;
use crate::renderer::Renderer;
use crate::state::State;
use crate::texture::Textures;
use crate::util::Vec2d;

pub struct App<'a> {
    pub renderer: Option<Renderer<'a>>,
    size: crate::util::Size,
    textures: Option<Textures>,
    state: Arc<Mutex<State>>,
    animations: Arc<Mutex<Vec<Mutex<Box<dyn Animation>>>>>,
    continue_receiving: Arc<AtomicBool>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let size = crate::util::Size { w: 1280, h: 720 };
        let state = State::new(size);
        let state_mutex = Arc::new(Mutex::new(state));
        App {
            renderer: None,
            size: size,
            textures: None,
            state: state_mutex,
            continue_receiving: Arc::new(AtomicBool::new(true)),
            animations: Default::default(),
        }
    }

    pub fn renderer(&self) -> &Renderer<'a> {
        self.renderer.as_ref().unwrap()
    }

    fn send_images_to_camera(
        is_running: Arc<AtomicBool>,
        receiver: Receiver<Vec<u8>>,
        mut camera: Camera,
    ) {
        while is_running.load(Ordering::Relaxed) {
            match receiver.recv() {
                Ok(img) => camera.send(&img).unwrap(),
                Err(_) => break,
            }
        }
    }

    fn animations_thread(
        animations: Arc<Mutex<Vec<Mutex<Box<dyn Animation>>>>>,
        is_running: Arc<AtomicBool>,
        state: Arc<Mutex<State>>,
    ) {
        let frame_time = Duration::from_millis(1000 / 20);
        while is_running.load(Ordering::Relaxed) {
            let now = Instant::now();
            let s = state.lock().unwrap();
            animations.lock().unwrap().iter_mut().for_each(|animation| {
                animation.lock().unwrap().advance_animation(&s);
            });
            drop(s);
            let elapsed = now.elapsed();
            if elapsed > frame_time {
                println!(
                    "frame budget overrun by {}",
                    (elapsed - frame_time).as_millis()
                );
            } else if (frame_time - elapsed).as_millis() > 5 {
                sleep(frame_time - elapsed);
            }
        }
    }

    fn microphone_thread(is_running: Arc<AtomicBool>, state: Arc<Mutex<State>>) {
        microphone::listen_to_microphone(is_running, state);
    }

    fn create_animations(&mut self, textures: &Textures) {
        let oiia_animation = ImageAnim::new(&textures.rotation, Vec2d { x: 300, y: 400 });
        let oiia_bounce_animation = Bounce::new(Box::new(oiia_animation));
        self.animations = Arc::new(Mutex::new(vec![Mutex::new(Box::new(
            oiia_bounce_animation,
        ))]));
    }

    fn launch_animations(&mut self, receiver: Receiver<Vec<u8>>) {
        let continue_receiving = self.continue_receiving.clone();
        let state = self.state.clone();
        let size = self.size.clone();
        let animations = self.animations.clone();
        spawn_camera_thread(receiver, continue_receiving.clone(), size);
        spawn_animations_thread(animations, continue_receiving.clone(), state.clone());
        spawn_microphone_thread(continue_receiving, state);
    }
}

fn spawn_camera_thread(
    receiver: Receiver<Vec<u8>>,
    continue_receiving: Arc<AtomicBool>,
    size: crate::util::Size,
) {
    thread::spawn(move || {
        let fps = 60.0;
        let camera = Camera::builder(
            u32::try_from(size.w).unwrap(),
            u32::try_from(size.h).unwrap(),
            f64::from(fps),
        )
        .format(PixelFormat::RGBA)
        .build()
        .unwrap();
        App::send_images_to_camera(continue_receiving, receiver, camera);
    });
}

fn spawn_animations_thread(
    animations: Arc<Mutex<Vec<Mutex<Box<dyn Animation>>>>>,
    continue_receiving: Arc<AtomicBool>,
    state: Arc<Mutex<State>>,
) {
    thread::spawn(move || {
        App::animations_thread(animations, continue_receiving, state);
    });
}

fn spawn_microphone_thread(continue_receiving: Arc<AtomicBool>, state: Arc<Mutex<State>>) {
    thread::spawn(move || {
        App::microphone_thread(continue_receiving, state);
    });
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
        let (sender, receiver) = sync_channel::<Vec<u8>>(2);
        let renderer_fut = Renderer::new(event_loop.owned_display_handle(), window.clone(), sender);
        let (renderer, textures) = pollster::block_on(renderer_fut);
        self.renderer = Some(renderer);
        self.create_animations(&textures);
        self.textures = Some(textures);
        self.launch_animations(receiver);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let renderer = self.renderer.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");

                self.continue_receiving.store(false, Ordering::Relaxed);
                renderer.exit();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let state = self.state.lock().unwrap();
                let anims = self.animations.lock().unwrap();
                let objs = anims
                    .iter()
                    .flat_map(|it| it.lock().unwrap().objects(&state))
                    .collect::<Vec<_>>();
                renderer.render(objs);
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }
            _ => (),
        }
    }
}
