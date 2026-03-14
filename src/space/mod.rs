mod config;
mod context;

pub use config::*;
pub use context::*;

use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{backend::LumaBackend, ui::LumaUI};

pub trait LumaHandler {
    fn configs() -> LumaWindowConfigs {
        LumaWindowConfigs::default()
    }

    fn rerender(&mut self, ui: &LumaUI, renderer: &mut LumaBackend);

    ///Executes when the provided `event`, is received by the window
    fn on_event(
        &mut self,
        _event: LumaEvent,
        window: &Window,
        ui: &LumaUI,
        renderer: &mut LumaBackend,
    );
}

#[derive(Debug)]
pub enum LumaEvent {
    Window(WindowEvent),
    Created,
    Suspended,
    Exiting,
    #[cfg(target_arch = "wasm32")]
    LumaContext(LumaContextInner),
}

pub struct LumaSpace<H>
where
    H: LumaHandler,
{
    context: LumaContext<H>,
}

impl<H> LumaSpace<H>
where
    H: LumaHandler + 'static,
{
    pub fn new(handler: H) -> Self {
        Self {
            context: LumaContext::new(handler),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn initialize(&mut self) {
        let lp: EventLoop<LumaEvent> = EventLoop::with_user_event().build().unwrap();

        let configs = H::configs();
        if configs.wait_for_events {
            lp.set_control_flow(ControlFlow::Wait);
        } else {
            lp.set_control_flow(ControlFlow::Poll);
        }

        if let Err(e) = lp.run_app(&mut self.context) {
            tracing::info!("{e:?}");
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn initialize(&mut self) {
        let lp: EventLoop<LumaEvent> = EventLoop::with_user_event().build().unwrap();

        self.context.proxy = Some(lp.create_proxy());

        let configs = H::configs();
        if configs.wait_for_events {
            lp.set_control_flow(ControlFlow::Wait);
        } else {
            lp.set_control_flow(ControlFlow::Poll);
        }

        if let Err(e) = lp.run_app(&mut self.context) {
            tracing::info!("{e:?}");
        }
    }
}
