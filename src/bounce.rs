use std::sync::MutexGuard;

use crate::animation::Animation;
use crate::object::Object;
use crate::resources::{self};
use crate::state::State;
use crate::texture::Textures;
use crate::util::{BoundingBox, Size, Vec2d};

pub struct Bounce {
    x_dir: i32,
    y_dir: i32,
    pos: Vec2d,
    inner: Box<dyn Animation>,
}

impl Bounce {
    pub fn new(inner: Box<dyn Animation>) -> Bounce {
        Bounce {
            x_dir: 20,
            y_dir: 20,
            pos: Vec2d { x: 0, y: 0 },
            inner: inner,
        }
    }
}

impl Animation for Bounce {
    fn bounding_box(&self) -> BoundingBox {
        self.inner.bounding_box().offset(&self.pos)
    }

    fn advance_animation(&mut self, state: &State) {
        self.inner.advance_animation(state);
        if !state.microphone_input_detected {
            return;
        }
        let new_pos = self.pos.plus(&Vec2d {
            x: self.x_dir,
            y: self.y_dir,
        });
        let bb = self.bounding_box();
        let bb1 = self.inner.bounding_box();
        let bb_top_right = bb.top_right();
        let touches_left = bb.bottom_left.x <= 0;
        let touches_bottom = bb.bottom_left.y <= 0;
        let touches_right = bb_top_right.x >= state.size.w as i32;
        let touches_top = bb_top_right.y >= state.size.h as i32;

        if touches_left {
            self.x_dir = self.x_dir.abs();
        }
        if touches_right {
            self.x_dir = -self.x_dir.abs();
        }
        if touches_top {
            self.y_dir = -self.y_dir.abs();
        }
        if touches_bottom {
            self.y_dir = self.y_dir.abs();
        }
        self.pos = new_pos;
    }

    fn objects(&self, state: &State) -> Vec<Object> {
        let inner_objects = self.inner.objects(state);
        let mut result: Vec<Object> = Vec::new();
        for obj in inner_objects {
            result.push(Object {
                position: obj.position.plus(&self.pos),
                texture: obj.texture,
            });
        }
        return result;
    }
}
