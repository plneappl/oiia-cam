use std::ops::Index;

use crate::{
    animation::Animation,
    object::Object,
    resources::Image,
    state::State,
    texture::{Texture, Textures},
    util::{BoundingBox, Vec2d},
};

pub struct ImageAnim {
    idx: usize,
    frames: Vec<Texture>,
    pos: Vec2d,
}

impl ImageAnim {
    pub fn new(frames: &Vec<Texture>, pos: Vec2d) -> Self {
        ImageAnim {
            idx: 0,
            frames: frames.clone(),
            pos: pos,
        }
    }
}

//cat_pos: Vec2d { x: 400, y: 400 },
//cat_img: &resources.standing,

impl Animation for ImageAnim {
    fn bounding_box(&self) -> BoundingBox {
        let mut result = self.frames[0].image.bounding_box(self.pos);
        for frame in self.frames.iter().skip(1) {
            let bb = frame.image.bounding_box(self.pos);
            result = result.plus(&bb);
        }
        return result;
    }

    fn advance_animation(&mut self, state: &State) {
        if state.microphone_input_detected {
            self.idx = (self.idx + 1) % self.frames.len();
        }
    }

    fn objects(&self, state: &State) -> Vec<Object> {
        let texture = if state.microphone_input_detected {
            self.frames[self.idx].clone()
        } else {
            self.frames[0].clone()
        };
        return vec![Object {
            position: self.pos,
            texture: texture,
        }];
    }
}
