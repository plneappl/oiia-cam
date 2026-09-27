use crate::{texture::Texture, util::Vec2d};

pub struct Object<'a> {
    pub position: Vec2d,
    pub texture: &'a Texture<'a>,
}
