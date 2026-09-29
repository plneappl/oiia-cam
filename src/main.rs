use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, Scope, ScopedJoinHandle, Thread, sleep};
use std::time::{Duration, Instant};

use image::{Rgb, Rgba};
use virtualcam::{Camera, PixelFormat};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::animation::Animation;
use crate::application::App;
use crate::bounce::Bounce;
use crate::imageanim::ImageAnim;
use crate::scene::build_scene;
use crate::state::State;
use crate::util::{Size, Vec2d};

mod animation;
mod application;
mod bounce;
mod imageanim;
mod microphone;
mod object;
mod renderer;
mod resources;
mod scene;
mod state;
mod texture;
mod toggleanim;
mod util;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
    println!("exiting...");
    drop(app);
    println!("done.");
}
