use std::sync::MutexGuard;

use crate::animation::Animation;
use crate::resources::{self, Resources};
use crate::state::State;
use crate::util::{Size, Vec2d};

pub struct Bounce<'a> {
    x_dir: i32,
    y_dir: i32,
    next_animation_frame: usize,
    resources: &'a Resources,
}

impl<'a> Bounce<'a> {
    pub fn new(resources: &'a Resources) -> Bounce {
        Bounce {
            x_dir: 20,
            y_dir: 20,
            next_animation_frame: 0,
            resources: resources,
        }
    }
}

impl<'a> Animation<'a> for Bounce<'a> {
    fn next_state(&mut self, state: &State<'a>) -> State<'a> {
        if !state.microphone_input_detected {
            return State {
                //cat_img: &self.resources.standing,
                ..*state
            };
        }

        let img_size_half = Size {
            w: state.cat_img.size.w / 2,
            h: state.cat_img.size.h / 2,
        };
        let new_pos = state.cat_pos.plus(&Vec2d {
            x: self.x_dir,
            y: self.y_dir,
        });
        let touches_left = new_pos.x - img_size_half.w as i32 <= 0;
        let touches_right =
            usize::try_from(new_pos.x + img_size_half.w as i32).unwrap_or(0) > state.size.w;
        let touches_top = new_pos.y - img_size_half.h as i32 <= 0;
        let touches_bottom =
            usize::try_from(new_pos.y + img_size_half.h as i32).unwrap_or(0) > state.size.h;

        if touches_left || touches_right {
            self.x_dir = -1 * self.x_dir
        }
        if touches_top || touches_bottom {
            self.y_dir = -1 * self.y_dir
        }
        let new_frame_idx = (self.next_animation_frame + 1) % self.resources.rotation.len();
        self.next_animation_frame = new_frame_idx;

        State {
            cat_pos: new_pos,
            cat_img: &self.resources.rotation[new_frame_idx],
            ..*state
        }
    }
}
