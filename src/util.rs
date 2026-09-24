#[derive(Copy, Clone)]
pub struct Size {
    pub w: usize,
    pub h: usize,
}

pub struct Vec2d {
    pub x: i32,
    pub y: i32,
}

impl Vec2d {
    pub fn plus(&self, other: &Vec2d) -> Vec2d {
        Vec2d {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    pub fn plus_x(&self, x_off: i32) -> Vec2d {
        Vec2d {
            x: self.x + x_off,
            y: self.y,
        }
    }
    pub fn plus_y(&self, y_off: i32) -> Vec2d {
        Vec2d {
            x: self.x,
            y: self.y + y_off,
        }
    }
}

pub struct Frame {
    pub buf: Vec<u8>,
    pub size: Size,
}
