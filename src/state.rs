use std::sync::Arc;

use crate::{
    object::Object,
    resources::{Image, Resources, place_image},
    util::{Frame, Size, Vec2d},
};

pub struct State<'a> {
    pub size: Size,
    pub microphone_input_detected: bool,
    pub cat_pos: Vec2d,
    pub cat_img: &'a Image,
}

impl<'a> State<'a> {
    pub fn new(resources: &'a Resources, size: Size) -> State<'a> {
        State {
            size: size,
            microphone_input_detected: false,
            cat_pos: Vec2d { x: 400, y: 400 },
            cat_img: &resources.standing,
        }
    }
    pub fn render(&self, frame: &mut Frame) -> () {
        place_image(&self.cat_img, frame, &self.cat_pos);
    }

    pub fn render_gpu(&self) -> Vec<Object> {
        vec![Object {
            position: self.cat_pos,
        }]
    }
}
