use crate::util::{Frame, Size};
use image::Rgb;
use virtualcam::{Camera, PixelFormat, VirtualCamError};

pub struct Scene {
    pub camera: Camera,
    pub size: Size,
    pub fps: u32,
}

pub fn build_scene(w: usize, h: usize, fps: u32) -> Result<Scene, VirtualCamError> {
    Ok(Scene {
        camera: Camera::builder(
            u32::try_from(w).unwrap(),
            u32::try_from(h).unwrap(),
            f64::from(fps),
        )
        .format(PixelFormat::RGB)
        .build()?,
        size: Size { w: w, h: h },
        fps: fps,
    })
}

impl Scene {
    pub fn blank(&mut self, background: Rgb<u8>) -> Frame {
        let mut frame = vec![0u8; self.size.w * self.size.h * 3];
        let mut col = 0;
        for i in frame.iter_mut() {
            *i = background[col];
            col += 1;
            col = col % 3;
        }
        return Frame {
            buf: frame,
            size: self.size,
        };
    }
}
