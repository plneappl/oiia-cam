use std::cmp::max;

use anyhow::*;
use image::GenericImageView;
use winit::dpi::PhysicalSize;

use crate::{
    resources::{self, Image},
    util::{Size, Vertex},
};

#[derive(Debug, Clone)]
pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub image: Image,
    pub index: u32,
}

pub struct Textures {
    pub standing: Texture,
    pub rotation: Vec<Texture>,
    pub popcat_closed: Texture,
    pub popcat_open: Texture,
}

const QUAD_INDICES: &[u32] = &[1, 0, 2, 0, 3, 2];

impl Texture {
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_size: PhysicalSize<u32>,
        texture_count: u32,
        img: Image,
        rgba: Vec<u8>,
        label: Option<&str>,
    ) -> Result<Self> {
        let dimensions = img.real_size;

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
        let vertices = Self::create_vertices(window_size, &img);

        Ok(Self {
            texture: texture,
            view: view,
            sampler: sampler,
            vertices: vertices,
            indices: QUAD_INDICES
                .iter()
                .map(|it| it + 4 * texture_count)
                .collect(),
            image: img,
            index: texture_count,
        })
    }

    fn create_vertices(window_size: PhysicalSize<u32>, img: &Image) -> Vec<Vertex> {
        let window_width = window_size.width as f64;
        let window_height = window_size.height as f64;
        let w = (-1.0 * (img.scaled_size.w as f64) / window_width) as f32;
        let h = (-1.0 * (img.scaled_size.h as f64) / window_height) as f32;

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

impl Textures {
    pub fn read_textures(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_size: PhysicalSize<u32>,
    ) -> Self {
        let mut count = 0;
        let mut get_texture = |(image, buf): (Image, Vec<u8>)| {
            let tex =
                Texture::from_image(device, queue, window_size, count, image, buf, None).unwrap();
            count += 1;
            tex
        };

        let standing_data = resources::read_standing();
        let standing_texture = get_texture(standing_data);
        let rotation = resources::read_spinning_animation();
        let mut rotation_textures = Vec::new();
        rotation_textures.reserve(rotation.len());
        for data in rotation {
            let tex = get_texture(data);
            rotation_textures.push(tex);
        }
        let popcat_closed = get_texture(resources::read_popcat_closed());
        let popcat_open = get_texture(resources::read_popcat_open());
        Textures {
            standing: standing_texture,
            rotation: rotation_textures,
            popcat_closed: popcat_closed,
            popcat_open: popcat_open,
        }
    }

    pub fn all_textures(&self) -> Vec<&Texture> {
        vec![
            vec![&self.standing],
            self.rotation.iter().collect::<Vec<&Texture>>(),
            vec![&self.popcat_closed, &self.popcat_open],
        ]
        .concat()
    }

    pub fn vertices_and_indices(&self) -> (Vec<Vertex>, Vec<u32>) {
        let all_texs = self.all_textures();
        let vertices = all_texs
            .iter()
            .flat_map(|it| it.vertices.clone())
            .collect::<Vec<Vertex>>();
        let indices = all_texs
            .iter()
            .flat_map(|it| it.indices.clone())
            .collect::<Vec<u32>>();
        return (vertices, indices);
    }
}
