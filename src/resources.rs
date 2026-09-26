use crate::util::{Frame, Size, Vec2d};
use image::{ImageError, open};

pub struct Image {
    pub buf: Vec<u8>,
    pub size: Size,
}

pub struct Resources {
    pub standing: Image,
    pub rotation: Vec<Image>,
}

pub fn read_resources<'a>() -> Result<Resources, ImageError> {
    let standing = open("cat.png")?.into_rgba8();
    let size = Size {
        w: usize::try_from(standing.width()).unwrap(),
        h: usize::try_from(standing.height()).unwrap(),
    };
    let buf = standing.into_vec();
    Ok(Resources {
        standing: Image {
            buf: buf,
            size: size,
        },
        rotation: Vec::new(),
    })
}

pub fn place_image(img: &Image, frame: &mut Frame, pos: &Vec2d) -> () {
    let w_img = img.size.w * 3;
    let h_img = img.size.h;
    let w_frame = frame.size.w * 3;
    let off_x = usize::try_from(pos.x).unwrap() * 3;
    let off_y = usize::try_from(pos.y).unwrap();
    for y_img in 0..h_img {
        let off_img = y_img * w_img;
        let y_frame = y_img + off_y;
        let off_frame = y_frame * w_frame + off_x;
        frame.buf.as_mut_slice()[off_frame..(off_frame + w_img)]
            .copy_from_slice(&img.buf.as_slice()[off_img..(off_img + w_img)]);
    }
}
