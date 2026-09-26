use crate::util::{Frame, Size};
use image::{Rgb, Rgba};
use virtualcam::{Camera, PixelFormat, VirtualCamError};

pub struct Scene {
    pub size: Size,
    pub fps: u32,
}

pub fn build_scene(size: Size, fps: u32) -> Result<(Scene, Camera), VirtualCamError> {
    Ok((
        Scene {
            size: size,
            fps: fps,
        },
        Camera::builder(
            u32::try_from(size.w).unwrap(),
            u32::try_from(size.h).unwrap(),
            f64::from(fps),
        )
        .format(PixelFormat::RGBA)
        .build()?,
    ))
}

impl Scene {
    pub fn blank(&mut self, background: Rgba<u8>) -> Frame {
        let mut frame = vec![0u8; self.size.w * self.size.h * 4];
        let mut col = 0;
        for i in frame.iter_mut() {
            *i = background[col];
            col += 1;
            col = col % 4;
        }
        return Frame {
            buf: frame,
            size: self.size,
        };
    }
}
