use crate::{
    backend::{BasicMaterial, LumaBackend, LumaShader, LumaVertex},
    mesh::{Mesh, MeshGeometry},
};
use luma_macros::shader;
shader! {
    Triangle {
        struct Vertex {
            #[location(0)] position: Vec2<f32>,
            #[location(1)] color: Vec3<f32>,
        }

        struct VertexResult {
           #[builtin(position)]
           position: Vec4<f32>
        }

       #[vertex]
       fn vertex_main(vertex: Vertex) -> VertexResult {
           VertexResult {
               position: Vec4(vertex.position.x, vertex.position.y, 0.0, 1.0)
           }
       }

       #[fragment]
       fn fragment_main(vertex: VertexResult) -> Vec4<f32> {
           Vec4(1.0, 1.0, 0.0, 1.0)
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
                            color: [1.0, 0.0, 0.0],
                        },
                        TriangleVertex {
                            position: [0.5, 0.5],
                            color: [0.0, 1.0, 0.0],
                        },
                        TriangleVertex {
                            position: [-0.5, 0.5],
                            color: [0.0, 0.0, 1.0],
                        },
                    ],
                    backend,
                ),
                BasicMaterial::new(backend.create_pipeline_builder_for::<Triangle>()),
            ),
        }
    }
}
