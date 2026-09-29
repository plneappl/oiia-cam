use crate::{
    object::Object,
    state::State,
    texture::Texture,
    util::{BoundingBox, Vec2d},
};

pub trait Animation: Send {
    fn bounding_box(&self) -> BoundingBox;
    fn advance_animation(&mut self, state: &State);
    fn objects(&self, state: &State) -> Vec<Object>;
}

pub fn bounding_box(pos: Vec2d, frames: &Vec<Texture>) -> BoundingBox {
    let mut result = frames[0].image.bounding_box(pos);
    for frame in frames.iter().skip(1) {
        let bb = frame.image.bounding_box(pos);
        result = result.plus(&bb);
    }
    return result;
}
