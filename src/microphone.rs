use std::{
    mem::transmute,
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread::sleep,
    time::Duration,
};

use cgmath::num_traits::abs;
use cpal::{
    BufferSize, InputCallbackInfo, SampleFormat,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::state::State;

pub fn listen_to_microphone(is_running: Arc<AtomicBool>, state: Arc<Mutex<State>>) {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("No default input device available");
    let supported_config = device
        .supported_input_configs()
        .expect("no input configs available (1)")
        .filter(|it| it.sample_format() == SampleFormat::F32)
        .next()
        .expect("No input configs available (2)")
        .with_max_sample_rate();
    let state = unsafe { transmute::<_, Arc<Mutex<State>>>(state) };

    let stream = device
        .build_input_stream(
            supported_config.into(),
            move |data: &[f32], input_callback_info: &InputCallbackInfo| {
                let mean: f32 = abs_limited_mean_kahan(data);
                let mut s = state.lock().unwrap();
                let microphone_input_detected = f32::sqrt(mean) > 0.08;
                s.update_microphone(microphone_input_detected);
                drop(s);
            },
            move |err| {},
            None,
        )
        .unwrap();
    stream.play();
    while is_running.load(std::sync::atomic::Ordering::Relaxed) {
        sleep(Duration::from_millis(100));
    }
    stream.pause();
    drop(stream);
}

// https://en.wikipedia.org/wiki/Kahan_summation_algorithm
fn abs_limited_mean_kahan(data: &[f32]) -> f32 {
    let mut sum: f32 = 0.0;
    let mut c: f32 = 0.0;
    for i in data {
        // abs(i) instead of i
        let iabs = abs(*i).min(0.3);
        let y = iabs - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }

    return sum / data.len() as f32;
}
