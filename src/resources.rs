use crate::util::{BoundingBox, Frame, Instance, Size, Vec2d};
use image::{ImageError, load_from_memory};
use wgpu::Device;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Image {
    pub label: String,
    pub size: Size,
}

impl Image {
    pub fn bounding_box(&self, centered_at: Vec2d) -> BoundingBox {
        let size = self.size.to_vec2d();
        let half_size = size.div(2);
        BoundingBox {
            bottom_left: centered_at.minus(&half_size),
            size: self.size.to_vec2d(),
        }
    }
}

macro_rules! read_img {
    ($file:expr, $id:expr $(,)?) => {{
        let img = load_from_memory(include_bytes!($file))
            .unwrap()
            .into_rgba8();
        let size = Size {
            w: usize::try_from(img.width()).unwrap(),
            h: usize::try_from(img.height()).unwrap(),
        };
        let img_struct = Image {
            label: String::from($id),
            size: size,
        };
        (img_struct, img.into_vec())
    }};
}

pub fn read_standing() -> (Image, Vec<u8>) {
    read_img!("../resources/cat.png", "standing")
}

pub fn read_standing_rev() -> (Image, Vec<u8>) {
    read_img!("../resources/cat_rev.png", "cat_rev")
}
