use cgmath::Vector3;
use wgpu::Extent3d;
use winit::dpi::PhysicalSize;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

    pub fn to_vec2d(&self) -> Vec2d {
        Vec2d {
            x: self.w as i32,
            y: self.h as i32,
        }
    }
}

#[derive(Debug)]
pub struct BoundingBox {
    pub bottom_left: Vec2d,
    pub size: Vec2d,
}

impl BoundingBox {
    pub fn zero() -> Self {
        BoundingBox {
            bottom_left: Vec2d { x: 0, y: 0 },
            size: Vec2d { x: 0, y: 0 },
        }
    }

    pub fn top_right(&self) -> Vec2d {
        self.bottom_left.plus(&self.size)
    }

    pub fn offset(&self, off: &Vec2d) -> BoundingBox {
        BoundingBox {
            bottom_left: self.bottom_left.plus(off),
            size: self.size,
        }
    }

    pub fn plus(&self, other: &BoundingBox) -> BoundingBox {
        let self_top_right = self.top_right();
        let other_top_right = other.top_right();
        let bottom_left = Vec2d {
            x: self.bottom_left.x.min(other.bottom_left.x),
            y: self.bottom_left.y.min(other.bottom_left.y),
        };
        let top_right = Vec2d {
            x: self_top_right.x.max(other_top_right.x),
            y: self_top_right.y.max(other_top_right.y),
        };
        BoundingBox {
            bottom_left: bottom_left,
            size: top_right.minus(&bottom_left),
        }
    }

    pub fn contains(self, point: &Vec2d) -> bool {
        let x_contains =
            self.bottom_left.x <= point.x && point.x <= self.bottom_left.x + self.size.x;
        let y_contains =
            self.bottom_left.y <= point.y && point.y <= self.bottom_left.y + self.size.y;
        x_contains && y_contains
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vec2d {
    pub x: i32,
    pub y: i32,
}

impl Vec2d {
    pub fn to_screen(&self, screen_size: PhysicalSize<u32>) -> Vector3<f32> {
        Vector3 {
            x: 2.0 * (self.x as f32) / (screen_size.width as f32) - 1.0,
            y: 2.0 * (self.y as f32) / (screen_size.height as f32) - 1.0,
            z: 0.0,
        }
    }

    pub fn plus(&self, other: &Vec2d) -> Vec2d {
        Vec2d {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn minus(&self, other: &Vec2d) -> Vec2d {
        Vec2d {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn div(&self, divisor: i32) -> Vec2d {
        Vec2d {
            x: self.x / divisor,
            y: self.y / divisor,
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

#[derive(Debug, Clone)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
