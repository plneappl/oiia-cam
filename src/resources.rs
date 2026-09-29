use crate::util::{BoundingBox, Frame, Instance, Size, Vec2d};
use image::{ImageError, load_from_memory};
use wgpu::Device;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Image {
    pub real_size: Size,
    pub scaled_size: Size,
}

impl Image {
    pub fn bounding_box(&self, centered_at: Vec2d) -> BoundingBox {
        let size = self.scaled_size.to_vec2d();
        let half_size = size.div(2);
        BoundingBox {
            bottom_left: centered_at.minus(&half_size),
            size: size,
        }
    }
}

macro_rules! read_img {
    ($file:expr, $scale:expr $(,)?) => {{
        let img = load_from_memory(include_bytes!($file))
            .unwrap()
            .into_rgba8();
        let real_size = Size {
            w: (img.width()) as usize,
            h: (img.height()) as usize,
        };
        let scaled_size = Size {
            w: (img.width() as f32 * $scale) as usize,
            h: (img.height() as f32 * $scale) as usize,
        };
        let img_struct = Image {
            scaled_size: scaled_size,
            real_size: real_size,
        };
        (img_struct, img.into_vec())
    }};
}

pub fn read_standing() -> (Image, Vec<u8>) {
    read_img!("../resources/cat.png", 1.0)
}

pub fn read_standing_rev() -> (Image, Vec<u8>) {
    read_img!("../resources/cat_rev.png", 1.0)
}

pub fn read_popcat_closed() -> (Image, Vec<u8>) {
    read_img!("../resources/popcat_1.png", 0.25)
}

pub fn read_popcat_open() -> (Image, Vec<u8>) {
    read_img!("../resources/popcat_2.png", 0.25)
}
