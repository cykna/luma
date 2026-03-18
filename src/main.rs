use crate::{
    backend::{LumaBackend, LumaShader},
    mesh::Mesh,
    space::{LumaEvent, LumaHandler, LumaSpace, LumaWindowConfigs},
    triangle::{Triangle, TriangleMesh},
    ui::LumaUI,
};

pub use vello::wgpu;
use winit::window::Window;
mod backend;
mod mesh;
mod space;
mod triangle;
mod ui;

#[cfg(target_arch = "wasm32")]
pub type Result<T> = anyhow::Result<T>;
#[cfg(target_arch = "wasm32")]
pub type Report = anyhow::Error;
#[cfg(not(target_arch = "wasm32"))]
pub type Result<T> = color_eyre::eyre::Result<T>;
#[cfg(not(target_arch = "wasm32"))]
pub type Report = color_eyre::Report;

#[cfg(not(target_arch = "wasm32"))]
fn init_logging() {
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    let fmt_layer = fmt::layer().with_target(false); // log bonito no terminal

    tracing_subscriber::registry().with(fmt_layer).init();
}
#[cfg(target_arch = "wasm32")]
fn init_logging() {
    use tracing_subscriber::prelude::*;
    use tracing_wasm::WASMLayer;

    let wasm_layer = WASMLayer::new(tracing_wasm::WASMLayerConfig::default());
    tracing_subscriber::registry().with(wasm_layer).init();
}
pub struct BasicHandler {
    meshes: Vec<TriangleMesh>,
}

impl LumaHandler for BasicHandler {
    fn configs() -> space::LumaWindowConfigs {
        LumaWindowConfigs::default()
    }

    fn rerender(&mut self, ui: &LumaUI, renderer: &mut LumaBackend) {
        renderer.render_ui(ui).unwrap();
        renderer.render(true).unwrap();
    }

    fn on_event(
        &mut self,
        event: space::LumaEvent,
        window: &Window,
        ui: &LumaUI,
        renderer: &mut LumaBackend,
    ) {
        match event {
            LumaEvent::Window(e) => match e {
                winit::event::WindowEvent::Resized(size) => {
                    renderer.resize(size.width, size.height)
                }
                _ => {}
            },

            LumaEvent::Created => {
                self.meshes.push(TriangleMesh::new(renderer));
                tracing::info!("Initialized Luma");
            }
            _ => {}
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    init_logging();
    let mut space = LumaSpace::new(BasicHandler { meshes: Vec::new() });
    space.initialize();
}

#[cfg(target_arch = "wasm32")]
pub fn main() {
    init_logging();
    console_error_panic_hook::set_once();
    let mut space = LumaSpace::new(BasicHandler {});
    space.initialize();
}
