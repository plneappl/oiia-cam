use wgpu::Extent3d;

#[derive(Copy, Clone)]
pub struct Size {
    pub w: usize,
    pub h: usize,
}

impl Size {
    pub fn to_extend3d(&self) -> Extent3d {
        Extent3d {
            width: u32::try_from(self.w).unwrap(),
            height: u32::try_from(self.h).unwrap(),
            depth_or_array_layers: 1,
        }
    }
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
