use bytemuck::Pod;
use vello::wgpu;

pub trait LumaShader {
    type Vertex: LumaVertex;
    type Result;
    const WGSL: &'static str;
    const SHADER_NAME: &'static str;
}

pub trait LumaVertex: Pod {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a>;
}
