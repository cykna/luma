use vello::wgpu::BufferUsages;

use crate::backend::{DynamicBuffer, LumaBackend, LumaVertex, Material};

pub struct MeshGeometry {
    innerbuffer: DynamicBuffer,
}
pub struct Mesh {
    geometry: MeshGeometry,
    material: Box<dyn Material>,
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
