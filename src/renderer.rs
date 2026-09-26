use std::{
    ops::RangeBounds,
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
};

use raw_window_handle::{DisplayHandle, RawDisplayHandle, RawWindowHandle};
use wgpu::{
    Backends, BufferUsages, Device, ExperimentalFeatures, Instance, InstanceDescriptor, Queue,
    RequestAdapterOptions, Surface, TextureFormat, TextureUsages,
    wgc::{device::queue::QueueSubmitError::CommandEncoder, global::Global, id::Id},
    wgt::WgpuHasDisplayHandle,
};
use winit::{dpi::Size, event_loop::OwnedDisplayHandle, window::Window};

pub struct Renderer<'a> {
    wgpu_instance: Instance,
    window: Arc<Window>,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    surface_format: TextureFormat,
    size: winit::dpi::PhysicalSize<u32>,
    images_sender: &'a Sender<Vec<u8>>,
}

impl<'a> Renderer<'a> {
    pub async fn new(
        display: OwnedDisplayHandle,
        window: Arc<Window>,
        images_sender: &'a Sender<Vec<u8>>,
    ) -> Renderer<'a> {
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
        let cap = surface.get_capabilities(&adapter);
        let size = window.inner_size();
        Renderer {
            wgpu_instance: inst,
            window: window,
            surface: surface,
            device: device,
            queue: queue,
            surface_format: cap.formats[0],
            size: size,
            images_sender: images_sender,
        }
    }

    pub fn render(&mut self) {
        println!("in render");
        self.draw();
    }

    pub fn draw(&mut self) {
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
                unreachable!("No error scope registered, so validation errors will panic")
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
        let mut encoder2 = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        encoder2.copy_texture_to_buffer(
            surface_texture.texture.as_image_copy(),
            buffer_info,
            wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
        );

        // If you wanted to call any drawing commands, they would go here.
        // End the renderpass.
        drop(renderpass);

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
                    images.send(buf.to_vec()).unwrap();
                }
            });
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            format: self.surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }
}
