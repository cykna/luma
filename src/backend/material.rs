use std::sync::Arc;

use vello::wgpu::{
    self, MultisampleState, PipelineCompilationOptions, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, VertexState,
};

///Implementations for the backend related to materials
use crate::backend::{LumaBackend, LumaShader, LumaVertex};

pub trait Material {
    ///The pipeline used by this material to render some mesh
    fn pipeline(&self) -> Arc<RenderPipeline>;
}

pub struct BasicMaterial {
    pipeline: Arc<RenderPipeline>,
}

impl BasicMaterial {
    pub fn new(pipeline: Arc<RenderPipeline>) -> Self {
        Self { pipeline }
    }
}

impl Material for BasicMaterial {
    fn pipeline(&self) -> Arc<RenderPipeline> {
        self.pipeline.clone()
    }
}

pub struct RenderPipelineBuilder<'a> {
    backend: &'a mut LumaBackend,
    name: &'static str,
    shader_layout: Option<wgpu::VertexBufferLayout<'a>>,
    render_method: PrimitiveTopology,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn with_render_method(&'a mut self, render: PrimitiveTopology) -> &'a mut Self {
        self.render_method = render;
        self
    }

    pub fn with_shader<S: LumaShader>(&'a mut self) -> &'a mut Self {
        self.backend.create_shader::<S>();
        self.shader_layout = Some(<S::Vertex as LumaVertex>::layout());
        self
    }

    ///Registers a new render pipeline on the backend and returns it
    pub fn build(self) -> Arc<RenderPipeline> {
        if let Some(pipeline) = self.backend.pipelines.get(self.name) {
            return pipeline.clone();
        }
        let shader = self
            .backend
            .shaders
            .get(self.name)
            .expect("Shader should be provided during pipeline creation");
        let out = Arc::new(
            self.backend
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some(self.name),
                    layout: None,
                    vertex: VertexState {
                        module: shader,
                        entry_point: Some("vertex_main"),
                        compilation_options: vello::wgpu::PipelineCompilationOptions {
                            constants: &[],
                            zero_initialize_workgroup_memory: true,
                        },
                        buffers: &[self.shader_layout.unwrap()],
                    },
                    primitive: PrimitiveState {
                        topology: self.render_method,
                        strip_index_format: Some(wgpu::IndexFormat::Uint32),
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: true,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: true,
                    },
                    depth_stencil: None,
                    multisample: MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: shader,
                        entry_point: Some("fragment_main"),
                        compilation_options: PipelineCompilationOptions {
                            constants: &[],
                            zero_initialize_workgroup_memory: true,
                        },
                        targets: &[],
                    }),
                    multiview: None,
                    cache: None,
                }),
        );
        self.backend.pipelines.insert(self.name, out.clone());
        out
    }
}

impl LumaBackend {
    pub fn create_shader<T: LumaShader>(&mut self) -> &ShaderModule {
        if !self.shaders.contains_key(T::SHADER_NAME) {
            let shader = self.device.create_shader_module(ShaderModuleDescriptor {
                label: Some(T::SHADER_NAME),
                source: vello::wgpu::ShaderSource::Wgsl(T::WGSL.into()),
            });
            self.shaders.insert(T::SHADER_NAME, shader);
        }
        self.shaders.get(T::SHADER_NAME).unwrap()
    }
    pub fn create_render_pipeline_builder<'a>(
        &'a mut self,
        name: &'static str,
    ) -> RenderPipelineBuilder<'a> {
        RenderPipelineBuilder {
            name,
            backend: self,
            shader_layout: None,
            render_method: PrimitiveTopology::TriangleList,
        }
    }
    pub fn create_pipeline_builder_for<T: LumaShader>(&mut self) -> Arc<RenderPipeline> {
        RenderPipelineBuilder {
            backend: self,
            name: T::SHADER_NAME,
            shader_layout: Some(<T::Vertex as LumaVertex>::layout()),
            render_method: PrimitiveTopology::TriangleList,
        }
        .build()
    }
}
