use crate::space::{LumaContext, LumaEvent, LumaHandler, LumaSpace, LumaWindowConfigs};
pub use vello::wgpu;
use winit::event::WindowEvent;
mod backend;

mod space;
mod ui;

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
    context: LumaContext<()>,
}

impl LumaHandler for BasicHandler {
    type Event = ();
    fn configs() -> space::LumaWindowConfigs {
        LumaWindowConfigs::default()
    }
    fn get_context(&self) -> &space::LumaContext<Self::Event> {
        &self.context
    }
    fn get_context_mut(&mut self) -> &mut space::LumaContext<Self::Event> {
        &mut self.context
    }
    fn on_event(&mut self, event: space::LumaEvent<Self::Event>) {
        match event {
            LumaEvent::Window(e) => match e {
                WindowEvent::RedrawRequested => {}
                _ => {}
            },
            LumaEvent::Created => {
                tracing::info!("Initialized Luma");
            }
            _ => {}
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    init_logging();
    let mut space = LumaSpace::new(BasicHandler {
        context: LumaContext::default(),
    });
    space.initialize().await;
}
#[cfg(target_arch = "wasm32")]
pub fn main() {
    init_logging();
    console_error_panic_hook::set_once();
    let mut space = LumaSpace::new(BasicHandler {
        context: LumaContext::default(),
    });
    wasm_bindgen_futures::spawn_local(async move { space.initialize().await });
}
