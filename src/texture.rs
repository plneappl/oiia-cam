use std::cmp::max;

use anyhow::*;
use image::GenericImageView;
use winit::dpi::PhysicalSize;

use crate::{
    resources::Image,
    util::{Size, Vertex},
};

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub vertices: Vec<Vertex>,
}

impl Texture {
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_size: PhysicalSize<u32>,
        img: &Image,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = &img.buf;
        let dimensions = img.size;

        let size = dimensions.to_extend3d();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let vertices = Self::create_vertices(window_size, img);

        Ok(Self {
            texture: texture,
            view: view,
            sampler: sampler,
            vertices: vertices,
        })
    }

    pub fn create_vertices(window_size: PhysicalSize<u32>, img: &Image) -> Vec<Vertex> {
        let window_width = window_size.width as f64;
        let window_height = window_size.height as f64;
        let w = (-1.0 * (img.size.w as f64) / window_width) as f32;
        let h = (-1.0 * (img.size.h as f64) / window_height) as f32;

        vec![
            Vertex {
                position: [-w, -h, 0.0],
                tex_coords: [1.0, 0.0],
            }, // A
            Vertex {
                position: [-w, h, 0.0],
                tex_coords: [1.0, 1.0],
            }, // B
            Vertex {
                position: [w, h, 0.0],
                tex_coords: [0.0, 1.0],
            }, // C
            Vertex {
                position: [w, -h, 0.0],
                tex_coords: [0.0, 0.0],
            }, // D
        ]
    }
}
