use crate::{
    texture::Texture,
    util::{BoundingBox, Vec2d},
};

#[derive(Clone, Debug)]
pub struct Object {
    pub position: Vec2d,
    pub texture: Texture,
}

impl Object {
    fn bounding_box(&self) -> BoundingBox {
        let size = self.texture.image.size.to_vec2d();
        let half_size = size.div(2);
        BoundingBox {
            bottom_left: self.position.minus(&half_size),
            size: size,
        }
    }
}
