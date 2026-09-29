use std::sync::{
    Arc,
    mpsc::{Sender, SyncSender},
};

use cgmath::{Quaternion, Vector3, prelude::*};
use wgpu::{
    BindGroupLayout, Buffer, BufferUsages, Device, ExperimentalFeatures, InstanceDescriptor, Queue,
    RenderPass, RenderPipeline, RequestAdapterOptions, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages, util::DeviceExt,
};
use winit::{event_loop::OwnedDisplayHandle, window::Window};

use crate::{
    object::Object,
    texture::Textures,
    util::{Instance, InstanceRaw, Vertex},
};

pub struct Renderer<'a> {
    wgpu_instance: wgpu::Instance,
    window: Arc<Window>,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    surface_format: TextureFormat,
    size: winit::dpi::PhysicalSize<u32>,
    images_sender: SyncSender<Vec<u8>>,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_vertices: u32,
    num_indices: u32,
    texture_bind_group_layout: BindGroupLayout,
}

impl<'a> Renderer<'a> {
    pub async fn new(
        display: OwnedDisplayHandle,
        window: Arc<Window>,
        images_sender: SyncSender<Vec<u8>>,
    ) -> (Renderer<'a>, Textures) {
        let size = window.inner_size();
        let inst_descriptor = InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Some(Box::new(display)),
        };
        let inst = wgpu::Instance::new(inst_descriptor);
        let surface = inst.create_surface(window.clone()).unwrap();
        let adapter = inst
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: true,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: ExperimentalFeatures::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();
        let surface_format = TextureFormat::Rgba8UnormSrgb;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Srgb,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![surface_format],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.width,
            height: size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoNoVsync,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let textures = Textures::read_textures(&device, &queue, size);
        let (vertices, indices) = textures.vertices_and_indices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[Some(&texture_bind_group_layout)],
                immediate_size: 0,
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc()), Some(InstanceRaw::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let renderer = Renderer {
            wgpu_instance: inst,
            window: window,
            surface: surface,
            device: device,
            queue: queue,
            config: surface_config,
            surface_format: surface_format,
            size: size,
            images_sender: images_sender,
            render_pipeline: render_pipeline,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            num_vertices: vertices.len() as u32,
            num_indices: indices.len() as u32,
            texture_bind_group_layout: texture_bind_group_layout,
        };
        (renderer, textures)
    }

    pub fn render(&mut self, objects: Vec<Object>) {
        //println!("in render");
        self.draw(objects);
        self.window.request_redraw();
    }

    pub fn draw(&mut self, objects: Vec<Object>) {
        // Create texture view.
        // NOTE: We must handle Timeout because the surface may be unavailable
        // (e.g., when the window is occluded on macOS).
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                println!("error, we'll ignore it...");
                return;
                //unreachable!("No error scope registered, so validation errors will panic")
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self
                    .wgpu_instance
                    .create_surface(self.window.clone())
                    .unwrap();
                self.configure_surface();
                return;
            }
        };
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let buffer = self.device.create_buffer(&wgpu::wgt::BufferDescriptor {
            label: None,
            size: u64::from(self.size.width) * u64::from(self.size.height) * 4,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let buffer_size = buffer.size();
        let buffer_ref = Arc::new(buffer);
        let buffer_info = wgpu::TexelCopyBufferInfo {
            buffer: &buffer_ref.clone(),
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.size.width * 4),
                rows_per_image: Some(self.size.height),
            },
        };

        // If you wanted to call any drawing commands, they would go here.
        renderpass.set_pipeline(&self.render_pipeline);
        for object in objects {
            self.render_object(&mut renderpass, object);
        }
        // End the renderpass.
        drop(renderpass);

        let mut encoder2 = self.device.create_command_encoder(&Default::default());
        encoder2.copy_texture_to_buffer(
            surface_texture.texture.as_image_copy(),
            buffer_info,
            wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
        );
        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish(), encoder2.finish()]);
        self.window.pre_present_notify();
        self.queue.present(surface_texture);
        let images = self.images_sender.clone();
        buffer_ref
            .clone()
            .map_async(wgpu::MapMode::Read, 0..buffer_size, move |it| {
                if it.is_ok() {
                    let b = buffer_ref.clone();
                    it.unwrap();
                    let buf = b.get_mapped_range(0..b.size()).unwrap();
                    images.send(buf.to_vec());
                }
            });
    }

    fn render_object(&mut self, renderpass: &mut RenderPass, object: Object) {
        let position = object.position.to_screen(self.size);
        let (instances, instance_buffer) =
            Self::positions_to_instance_buffer(&self.device, &vec![position]);

        let diffuse_entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&object.texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&object.texture.sampler),
            },
        ];
        let diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: diffuse_entries.as_slice(),
            label: Some("diffuse_bind_group"),
        });

        renderpass.set_bind_group(0, &diffuse_bind_group, &[]);
        renderpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        renderpass.set_vertex_buffer(1, instance_buffer.slice(..));
        renderpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        renderpass.draw_indexed(
            (object.texture.index * 6)..(object.texture.index * 6 + 6),
            0,
            0..instances.len() as _,
        );
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    fn configure_surface(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    fn positions_to_instance_buffer(
        device: &Device,
        positions: &[Vector3<f32>],
    ) -> (Vec<Instance>, Buffer) {
        let instances = positions
            .iter()
            .map(|pos| Instance {
                position: *pos,
                rotation: Quaternion::from_axis_angle(pos.normalize(), cgmath::Deg(0.0)),
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(|it| it.to_raw()).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        (instances, instance_buffer)
    }

    pub fn exit(&mut self) {
        self.device.destroy();
        self.window.clone().set_visible(false);
    }
}
