use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::thread::{self, Thread, sleep};
use std::time::{Duration, Instant};

use image::{Rgb, Rgba};
use virtualcam::Camera;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::animation::Animation;
use crate::application::App;
use crate::bounce::Bounce;
use crate::resources::read_resources;
use crate::scene::build_scene;
use crate::state::State;
use crate::util::Size;

mod animation;
mod application;
mod bounce;
mod renderer;
mod resources;
mod scene;
mod state;
mod util;

fn send_images_to_camera(
    continue_receiving: Arc<AtomicBool>,
    receiver: Receiver<Vec<u8>>,
    mut camera: Camera,
) {
    while continue_receiving.load(Ordering::Relaxed) {
        match receiver.recv() {
            Ok(img) => camera.send(&img).unwrap(),
            Err(_) => break,
        }
    }
}

fn main() {
    let size = Size { w: 1280, h: 720 };
    let (mut the_scene, camera) = build_scene(size, 60).unwrap();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let (sender, receiver) = channel();
    let resources = read_resources().unwrap();
    let mut app = App::new(size, &resources, &sender);
    let continue_receiving_ref = Arc::new(AtomicBool::new(true));
    let continue_receiving = continue_receiving_ref.clone();
    let camera_thread = thread::spawn(move || {
        send_images_to_camera(continue_receiving_ref, receiver, camera);
    });
    event_loop.run_app(&mut app).unwrap();
    println!("exiting...");
    drop(app);
    drop(sender);
    continue_receiving.store(false, Ordering::Relaxed);
    camera_thread.join().unwrap();
    println!("done.");

    //let mut state = State::new(&resources, the_scene.size);
    //let mut animation = Bounce {
    //    x_dir: 10,
    //    y_dir: 10,
    //};
    //let frame_time = Duration::from_millis(1000 / u64::from(the_scene.fps));
    //loop {
    //    let now = Instant::now();
    //    let mut frame = the_scene.blank(Rgba([255, 255, 255, 255]));
    //
    //    state.render(&mut frame);
    //    state = animation.next_state(state);
    //}
}
