use flume::Sender;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::{application::ApplicationHandler, window::Window};

use crate::{
    backend::LumaBackend,
    space::{LumaEvent, LumaWindowConfigs},
    ui::LumaUI,
};

#[derive(Default)]
pub struct LumaContext<E: std::fmt::Debug> {
    ui: LumaUI,
    config: LumaWindowConfigs,
    window: Option<Window>,
    rendering_backend: Option<LumaBackend>,
    pub(crate) sender: Option<Sender<LumaEvent<E>>>,
}

impl<E: std::fmt::Debug> LumaContext<E> {
    pub fn sender(&self) -> &Sender<LumaEvent<E>> {
        if let Some(ref sender) = self.sender {
            sender
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"Ue".into());
            }
            unreachable!(
                "The context should be used only after executing initialize on a LumaSpace"
            );
        }
    }
    pub fn window(&self) -> &Window {
        if let Some(ref window) = self.window {
            window
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"Ue".into());
            }
            unreachable!(
                "The context should be used only after executing initialize on a LumaSpace"
            );
        }
    }
}

impl<E: 'static + std::fmt::Debug> ApplicationHandler<E> for LumaContext<E> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = self.config.to_window_attribs();
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{JsCast, UnwrapThrowExt};
            use winit::{platform::web::WindowAttributesExtWebSys, window::WindowAttributes};
            const CANVAS_ID: &str = "canvas";
            let mut window_attributes = self.config.to_window_attribs();
            let window = vello::wgpu::web_sys::window().expect("Window not found");
            let document = window.document().expect("Document not found");
            let canvas = document
                .get_element_by_id(CANVAS_ID)
                .expect("Canvas with id 'canvas' not found");
            let html_canvas_element = canvas.unchecked_into();
            let window_attributes =
                WindowAttributes::default().with_canvas(Some(html_canvas_element));
            self.window = Some(event_loop.create_window(window_attributes).unwrap());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.window = Some(event_loop.create_window(window_attributes).unwrap());
        }
        let e = self.sender().send(LumaEvent::Created);
        tracing::info!("Enviado o {e:?}");
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: E) {
        let _ = self.sender().send(LumaEvent::User(event));
    }
    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let _ = self.sender().send(LumaEvent::Window(event));
    }
}
