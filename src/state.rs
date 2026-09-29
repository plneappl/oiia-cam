use std::sync::Arc;

use crate::util::{Frame, Size, Vec2d};

pub struct State {
    pub size: Size,
    _microphone_input_detected: i16,
}

impl State {
    pub fn new(size: Size) -> State {
        State {
            size: size,
            _microphone_input_detected: 0,
        }
    }

    pub fn update_microphone(&mut self, input_detected: bool) {
        if input_detected {
            if self._microphone_input_detected == 0 {
                self._microphone_input_detected += 40;
            } else {
                self._microphone_input_detected += 20;
            }
        } else {
            self._microphone_input_detected -= 5;
        }
        self._microphone_input_detected = self._microphone_input_detected.min(100).max(0);
    }

    pub fn microphone_input_detected(&self, sensitivity: i16) -> bool {
        self._microphone_input_detected > sensitivity
    }
}
