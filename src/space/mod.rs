mod config;
mod context;

pub use config::*;
pub use context::*;

use std::sync::Arc;

use async_lock::RwLock;

use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};

pub trait LumaHandler {
    type Event: 'static + Send + Sync;
    fn configs() -> LumaWindowConfigs {
        LumaWindowConfigs::default()
    }

    fn get_context(&self) -> &LumaContext<Self::Event>;
    fn get_context_mut(&mut self) -> &mut LumaContext<Self::Event>;

    ///Executes when the provided `event`, is received by the window
    fn on_event(&mut self, _event: LumaEvent<Self::Event>);
}

pub enum LumaEvent<E> {
    Window(WindowEvent),
    User(E),
    Created,
    Suspended,
    Exiting,
}

pub struct LumaSpace<H>
where
    H: LumaHandler,
{
    handler: Arc<RwLock<H>>,
}

impl<H> LumaSpace<H>
where
    H: LumaHandler + 'static + Send + Sync,
{
    pub fn new(handler: H) -> Self {
        Self {
            handler: Arc::new(RwLock::new(handler)),
        }
    }

    pub async fn initialize(&mut self) {
        let lp: EventLoop<H::Event> = EventLoop::with_user_event().build().unwrap();
        let configs = H::configs();
        if configs.wait_for_events {
            lp.set_control_flow(ControlFlow::Wait);
        } else {
            lp.set_control_flow(ControlFlow::Poll);
        }
        let (tx, rx) = flume::bounded(128);

        let handler = self.handler.clone();
        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(async move {
            let handler = handler.clone();
            while let Ok(event) = rx.recv_async().await {
                handler.write().await.on_event(event);
            }
        });
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let handler = handler.clone();
                while let Ok(event) = rx.recv_async().await {
                    handler.write().await.on_event(event);
                }
            });
        }
        let mut handle = self.handler.write().await;
        let ctx = handle.get_context_mut();
        ctx.sender = Some(tx);
        lp.run_app(ctx).unwrap();
    }
}
