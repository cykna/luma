mod dynamic_buffer;
mod helpers;
mod material;
mod shader;

pub use dynamic_buffer::*;
pub use material::*;
pub use shader::*;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use vello::peniko::color::AlphaColor;
use vello::wgpu::wgt::{
    CommandEncoderDescriptor, DeviceDescriptor, TextureDescriptor, TextureViewDescriptor,
};
use vello::wgpu::{
    self, Adapter, BackendOptions, Backends, ExperimentalFeatures, Extent3d, Features, Instance,
    InstanceFlags, Limits, MemoryBudgetThresholds, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RequestAdapterOptions, ShaderModule,
    SurfaceConfiguration, Texture, Trace,
};
use vello::wgpu::{Device, Surface};
use vello::{RenderParams, Renderer, RendererOptions};
use winit::window::Window;

use crate::Result;
use crate::mesh::{LumaRenderable, Mesh};
use crate::ui::LumaUI;

pub struct LumaRenderingContext {
    surface: Surface<'static>,
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

pub struct LumaBackend {
    context: LumaRenderingContext,
    ui_renderer: Renderer,
    ui_texture: Texture,
    shaders: HashMap<&'static str, ShaderModule>,
    pipelines: HashMap<&'static str, Arc<RenderPipeline>>,
}

impl Deref for LumaBackend {
    type Target = LumaRenderingContext;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
impl DerefMut for LumaBackend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl std::fmt::Debug for LumaBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = f
            .debug_struct("LumaBackend")
            .field("ui_texture", &self.ui_texture)
            .field("ui_renderer", &"<Not Debuggable>")
            .field("surface", &self.surface)
            .field("instance", &self.instance)
            .field("adapter", &self.adapter)
            .field("device", &self.device)
            .field("queue", &self.queue)
            .finish()?;
        write!(f, "{:?}", out)
    }
}

impl LumaBackend {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::empty(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
        });
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("device and queue creation"),
                required_features: Features::DEPTH_CLIP_CONTROL
                    | Features::CONSERVATIVE_RASTERIZATION,
                required_limits: Limits::defaults(),
                experimental_features: ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: Trace::Off,
            })
            .await?;
        let Some(config) = surface.get_default_config(&adapter, size.width, size.height) else {
            return Err(crate::Report::msg(
                "Surface could not retrieve its default config",
            ));
        };
        surface.configure(&device, &config);

        Ok(Self {
            ui_texture: device.create_texture(&TextureDescriptor {
                label: Some("Ui texture creation"),
                size: Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &config.view_formats,
            }),
            ui_renderer: Renderer::new(&device, RendererOptions::default()).unwrap(),
            context: LumaRenderingContext {
                instance,
                adapter,
                device,
                queue,
                surface,
                config,
            },
            shaders: HashMap::new(),
            pipelines: HashMap::new(),
        })
    }

    ///Creates an texture of this backend with the current surface config
    pub fn create_surface_texture(&mut self) -> Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Ui texture resize"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm, // Mantenha o formato original
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &self.config.view_formats,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.ui_texture = self.create_surface_texture();
            tracing::info!("Resized para {}x{}", width, height);
        }
    }

    #[inline]
    ///Creates a new LumaUI that uses this backend to make vello render internally
    pub fn create_ui(&self) -> Result<LumaUI> {
        LumaUI::new(&self.device)
    }

    pub fn render_ui(&mut self, ui: &LumaUI) -> Result<()> {
        let width = self.config.width;
        let height = self.config.height;

        let texture = self.ui_texture.create_view(&TextureViewDescriptor {
            label: Some("ui texture view creation"),
            ..Default::default()
        });

        let result = self.ui_renderer.render_to_texture(
            &self.context.device,
            &self.context.queue,
            ui.scene(),
            &texture,
            &RenderParams {
                base_color: AlphaColor::from_rgb8(0, 0, 0),
                width,
                height,
                antialiasing_method: vello::AaConfig::Area,
            },
        );
        #[cfg(not(target_arch = "wasm32"))]
        result.map_err(|e| color_eyre::Report::new(e))?;
        self.device.poll(wgpu::PollType::Poll)?;

        Ok(())
    }

    pub fn render<R: LumaRenderable>(&mut self, meshes: &[R], merge_with_ui: bool) -> Result<()> {
        let frame = self.surface.get_current_texture()?;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let view = frame.texture.create_view(&TextureViewDescriptor {
                label: Some("frame texture"),
                ..Default::default()
            });
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render pass"),
                timestamp_writes: None,
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
            });
            for mesh in meshes {
                mesh.render(&mut render_pass);
            }
        };
        if merge_with_ui {
            //todo
        }
        self.queue.submit([encoder.finish()]);
        frame.present();

        Ok(())
    }
}
