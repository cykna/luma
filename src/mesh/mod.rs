use std::sync::Arc;

use vello::wgpu::{BufferSlice, BufferUsages, RenderPass, RenderPipeline};

use crate::backend::{DynamicBuffer, LumaBackend, LumaVertex, Material};

pub struct MeshGeometry {
    innerbuffer: DynamicBuffer,
}
pub struct Mesh {
    geometry: MeshGeometry,
    material: Box<dyn Material>,
}

impl Mesh {
    pub fn pipeline(&self) -> Arc<RenderPipeline> {
        self.material.pipeline()
    }
    pub fn vertex_count(&self) -> usize {
        self.geometry.innerbuffer.len()
    }
    pub fn vertex_slice(&self) -> BufferSlice {
        self.geometry.innerbuffer.inner.slice(..)
    }
}

pub trait LumaRenderable {
    fn mesh(&self) -> &Mesh;
    fn render(&self, pass: &mut RenderPass);
}

impl MeshGeometry {
    pub fn new<T: LumaVertex>(vertices: &[T], backend: &LumaBackend) -> Self {
        Self {
            innerbuffer: backend.create_dyn_buffer(vertices, BufferUsages::VERTEX),
        }
    }
}

impl Mesh {
    pub fn new<M: Material + 'static>(geometry: MeshGeometry, material: M) -> Self {
        Self {
            geometry,
            material: Box::new(material),
        }
    }
}
