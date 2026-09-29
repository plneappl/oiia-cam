use std::ops::Index;

use crate::{
    animation::{self, Animation},
    object::Object,
    resources::Image,
    state::State,
    texture::{Texture, Textures},
    util::{BoundingBox, Vec2d},
};

pub struct ToggleAnim {
    idx: usize,
    frames: Vec<Texture>,
    pos: Vec2d,
}

impl ToggleAnim {
    pub fn new(frames: &Vec<Texture>, pos: Vec2d) -> Self {
        ToggleAnim {
            idx: 0,
            frames: frames.clone(),
            pos: pos,
        }
    }
}

//cat_pos: Vec2d { x: 400, y: 400 },
//cat_img: &resources.standing,

impl Animation for ToggleAnim {
    fn bounding_box(&self) -> BoundingBox {
        animation::bounding_box(self.pos, &self.frames)
    }

    fn advance_animation(&mut self, state: &State) {
        if state.microphone_input_detected {
            self.idx = 1;
        } else {
            self.idx = 0;
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
