use std::thread::sleep;
use std::time::{Duration, Instant};

use image::Rgb;
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

fn main() {
    let size = Size { w: 1280, h: 720 };
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(size);
    event_loop.run_app(&mut app).unwrap();

    let mut the_scene = build_scene(size, 60).unwrap();
    let resources = read_resources().unwrap();
    let mut state = State::new(&resources, the_scene.size);
    let mut animation = Bounce {
        x_dir: 10,
        y_dir: 10,
    };
    let frame_time = Duration::from_millis(1000 / u64::from(the_scene.fps));
    loop {
        let now = Instant::now();
        let mut frame = the_scene.blank(Rgb([255, 255, 255]));

        state.render(&mut frame);
        state = animation.next_state(state);

        the_scene.camera.send(frame.buf.as_slice()).unwrap();
        let elapsed_time = now.elapsed();
        if elapsed_time > frame_time {
            let overrun = (elapsed_time - frame_time).as_millis();
            println!("Time frame broken by {overrun}");
        } else {
            let remaining = frame_time - elapsed_time;
            if remaining.as_millis() >= 5 {
                sleep(remaining);
            }
        }
    }
}
