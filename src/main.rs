use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, Thread, sleep};
use std::time::{Duration, Instant};

use image::{Rgb, Rgba};
use virtualcam::{Camera, PixelFormat};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::animation::Animation;
use crate::application::App;
use crate::bounce::Bounce;
use crate::resources::{Resources, read_resources};
use crate::scene::build_scene;
use crate::state::State;
use crate::util::{Size, Vec2d};

mod animation;
mod application;
mod bounce;
mod microphone;
mod object;
mod renderer;
mod resources;
mod scene;
mod state;
mod texture;
mod util;

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

fn state_thread(is_running: Arc<AtomicBool>, state: Arc<Mutex<State>>) {
    let mut animation = Bounce {
        x_dir: 10,
        y_dir: 10,
    };
    let frame_time = Duration::from_millis(1000 / 60);
    while is_running.load(Ordering::Relaxed) {
        let now = Instant::now();
        let mut s = state.lock().unwrap();
        let new_state = animation.next_state(&s);
        *s = new_state;
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

fn main() {
    let size = Size { w: 1280, h: 720 };
    let fps = 60.0;
    let camera = Camera::builder(
        u32::try_from(size.w).unwrap(),
        u32::try_from(size.h).unwrap(),
        f64::from(fps),
    )
    .format(PixelFormat::RGBA)
    .build()
    .unwrap();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let resources = read_resources().unwrap();
    let state = State::new(&resources, size);
    let state_mutex = Arc::new(Mutex::new(state));
    let continue_receiving = Arc::new(AtomicBool::new(true));
    let (sender, receiver) = sync_channel::<Vec<u8>>(2);
    thread::scope(|scope| {
        let mut app = App::new(size, &resources, state_mutex.clone(), sender);
        scope.spawn(|| {
            send_images_to_camera(continue_receiving.clone(), receiver, camera);
        });
        scope.spawn(|| {
            state_thread(continue_receiving.clone(), state_mutex.clone());
        });
        scope.spawn(|| {
            microphone_thread(continue_receiving.clone(), state_mutex.clone());
        });
        event_loop.run_app(&mut app).unwrap();
        println!("exiting...");
        continue_receiving.store(false, Ordering::Relaxed);
    });
    println!("done.");
}
