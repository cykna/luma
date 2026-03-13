use crate::space::{LumaContext, LumaEvent, LumaHandler, LumaSpace, LumaWindowConfigs};
pub use vello::wgpu;
use winit::event::WindowEvent;
mod backend;
mod space;
mod ui;

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
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let mut space = LumaSpace::new(BasicHandler {
        context: LumaContext::default(),
    });
    space.initialize().await;
}
