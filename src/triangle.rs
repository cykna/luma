use crate::{
    backend::{BasicMaterial, LumaBackend, LumaShader, LumaVertex, Material},
    mesh::{Mesh, MeshGeometry},
};
use luma_macros::shader;
use vello::wgpu;
shader! {
    Triangle {
        struct Vertex {
            position: Vec2<f32>
        }

        struct VertexResult {
           #[builtin(position)]
           position: Vec4<f32>
        }

       #[vertex]
       fn main(vertex: Vertex) -> VertexResult {
           VertexResult {
               position: Vec4(vertex.position.x, vertex.position.y, 0.0, 1.0)
           }
       }

       #[fragment]
       fn frag(vertex: VertexResult) -> Vec4<f32> {
           Vec4(1.0, 1.0, 0.0, 1.0)
       }
   }
}

impl LumaVertex for TriangleVertex {
    fn layout<'a>() -> vello::wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2,
            ],
        }
    }
}

pub struct TriangleMesh {
    mesh: Mesh,
}

impl TriangleMesh {
    pub fn new(backend: &mut LumaBackend) -> Self {
        Self {
            mesh: Mesh::new(
                MeshGeometry::new(
                    &[
                        TriangleVertex {
                            position: [0.0, -0.5],
                        },
                        TriangleVertex {
                            position: [0.5, 0.5],
                        },
                        TriangleVertex {
                            position: [-0.5, 0.5],
                        },
                    ],
                    backend,
                ),
                BasicMaterial::new(backend.create_pipeline_builder_for::<Triangle>()),
            ),
        }
    }
}
