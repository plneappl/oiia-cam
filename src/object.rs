use crate::{
    texture::Texture,
    util::{BoundingBox, Vec2d},
};

#[derive(Clone, Debug)]
pub struct Object {
    pub position: Vec2d,
    pub texture: Texture,
}
