use std::cmp::max;

use anyhow::*;
use image::GenericImageView;
use winit::dpi::PhysicalSize;

use crate::{
    resources::{Image, Resources},
    util::{Size, Vertex},
};

pub struct Texture<'a> {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub image: &'a Image,
}

pub struct Textures<'a> {
    pub standing: Texture<'a>,
    pub rotation: Vec<Texture<'a>>,
}

const QUAD_INDICES: &[u16] = &[1, 0, 2, 0, 3, 2];

impl<'a> Texture<'a> {
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_size: PhysicalSize<u32>,
        texture_count: u16,
        img: &'a Image,
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
            indices: QUAD_INDICES.iter().map(|it| it + texture_count).collect(),
            image: img,
        })
    }

    fn create_vertices(window_size: PhysicalSize<u32>, img: &Image) -> Vec<Vertex> {
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

impl<'a> Textures<'a> {
    pub fn fromResources(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_size: PhysicalSize<u32>,
        resources: &'a Resources,
    ) -> Self {
        let mut count = 0;
        let standing_texture =
            Texture::from_image(device, queue, window_size, count, &resources.standing, None)
                .unwrap();
        count += 1;
        let mut rotation_textures = Vec::new();
        rotation_textures.reserve(resources.rotation.len());
        for img in &resources.rotation {
            let tex = Texture::from_image(device, queue, window_size, count, &img, None).unwrap();
            rotation_textures.push(tex);
            count += 1;
        }
        Textures {
            standing: standing_texture,
            rotation: rotation_textures,
        }
    }

    pub fn all_textures(&self) -> Vec<&Texture> {
        vec![
            vec![&self.standing],
            self.rotation.iter().collect::<Vec<&Texture>>(),
        ]
        .concat()
    }

    pub fn vertices_and_indices(&self) -> (Vec<Vertex>, Vec<u16>) {
        let all_texs = self.all_textures();
        let vertices = all_texs
            .iter()
            .flat_map(|it| it.vertices.clone())
            .collect::<Vec<Vertex>>();
        let indices = all_texs
            .iter()
            .flat_map(|it| it.indices.clone())
            .collect::<Vec<u16>>();
        return (vertices, indices);
    }
}
