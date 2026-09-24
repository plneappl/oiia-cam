use crate::{
    resources::{Image, Resources, place_image},
    util::{Frame, Size, Vec2d},
};

pub struct State<'a> {
    pub size: Size,
    pub cat_pos: Vec2d,
    pub cat_img: &'a Image,
}

impl<'a> State<'a> {
    pub fn new(resources: &'a Resources, size: Size) -> State<'a> {
        State {
            size: size,
            cat_pos: Vec2d { x: 0, y: 0 },
            cat_img: &resources.standing,
        }
    }
    pub fn render(&self, frame: &mut Frame) -> () {
        place_image(&self.cat_img, frame, &self.cat_pos);
    }
}
