use crate::util::{Frame, Instance, Size, Vec2d};
use image::{ImageError, load_from_memory};
use wgpu::Device;

#[derive(PartialEq, Eq)]
pub struct Image {
    pub label: String,
    pub buf: Vec<u8>,
    pub size: Size,
}

pub struct Resources {
    pub standing: Image,
    pub rotation: Vec<Image>,
}

pub fn read_resources<'a>() -> Result<Resources, ImageError> {
    let standing = load_from_memory(include_bytes!("../resources/cat.png"))?.into_rgba8();
    let standing_rev = load_from_memory(include_bytes!("../resources/cat_rev.png"))?.into_rgba8();
    let size = Size {
        w: usize::try_from(standing.width()).unwrap(),
        h: usize::try_from(standing.height()).unwrap(),
    };
    let standing_img = Image {
        label: String::from("standing"),
        buf: standing.clone().into_vec(),
        size: size,
    };
    let standing_img2 = Image {
        label: String::from("anim1"),
        buf: standing.into_vec(),
        size: size,
    };
    let standing_rev_img = Image {
        label: String::from("anim2"),
        buf: standing_rev.into_vec(),
        size: size,
    };
    Ok(Resources {
        standing: standing_img,
        rotation: vec![standing_img2, standing_rev_img],
    })
}
