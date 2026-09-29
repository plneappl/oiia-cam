use std::sync::Arc;

use crate::util::{Frame, Size, Vec2d};

pub struct State {
    pub size: Size,
    pub microphone_input_detected: bool,
}

impl State {
    pub fn new(size: Size) -> State {
        State {
            size: size,
            microphone_input_detected: false,
        }
    }
}
