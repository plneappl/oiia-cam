use std::ops::Index;

use crate::{
    animation::{self, Animation},
    object::Object,
    resources::Image,
    state::State,
    texture::{Texture, Textures},
    util::{BoundingBox, Vec2d},
};

pub struct ImageAnim {
    idx: usize,
    still: Texture,
    frames: Vec<Texture>,
    pos: Vec2d,
    sensitivity: i16,
}

impl ImageAnim {
    pub fn new(still: &Texture, frames: &Vec<Texture>, pos: Vec2d, sensitivity: i16) -> Self {
        ImageAnim {
            idx: 0,
            still: still.clone(),
            frames: frames.clone(),
            pos: pos,
            sensitivity: sensitivity,
        }
    }
}

//cat_pos: Vec2d { x: 400, y: 400 },
//cat_img: &resources.standing,

impl Animation for ImageAnim {
    fn bounding_box(&self) -> BoundingBox {
        animation::bounding_box(self.pos, &self.frames)
    }

    fn advance_animation(&mut self, state: &State) {
        if state.microphone_input_detected(self.sensitivity) {
            self.idx = (self.idx + 1) % self.frames.len();
        }
    }

    fn objects(&self, state: &State) -> Vec<Object> {
        let texture = if state.microphone_input_detected(self.sensitivity) {
            self.frames[self.idx].clone()
        } else {
            self.still.clone()
        };
        return vec![Object {
            position: self.pos,
            texture: texture,
        }];
    }
}
