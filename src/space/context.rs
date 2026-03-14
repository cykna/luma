use std::sync::Arc;

use flume::Sender;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::{application::ApplicationHandler, window::Window};

use crate::{
    backend::LumaBackend,
    space::{LumaEvent, LumaHandler, LumaWindowConfigs},
    ui::LumaUI,
};

#[derive(Debug)]
pub(crate) struct LumaContextInner {
    ui: LumaUI,
    window: Arc<Window>,
    rendering_backend: LumaBackend,
}

pub struct LumaContext<T: LumaHandler> {
    handler: T,
    config: LumaWindowConfigs,

    ///Inner contents to render properly
    inner: Option<LumaContextInner>,

    #[cfg(target_arch = "wasm32")]
    pub(crate) proxy: Option<EventLoopProxy<LumaEvent>>,
}

impl<T: LumaHandler> LumaContext<T> {
    pub fn new(handler: T) -> Self {
        Self {
            handler,
            config: T::configs(),
            inner: None,
            #[cfg(target_arch = "wasm32")]
            proxy: None,
        }
    }
}

impl<T: LumaHandler> LumaContext<T> {
    pub fn ui(&self) -> &LumaUI {
        if let Some(ref inner) = self.inner {
            &inner.ui
        } else {
            unreachable!(
                "The context should be used only after executing initialize on a LumaSpace"
            );
        }
    }

    pub fn backend(&self) -> &LumaBackend {
        if let Some(ref inner) = self.inner {
            &inner.rendering_backend
        } else {
            unreachable!(
                "The context should be used only after executing initialize on a LumaSpace"
            );
        }
    }
    pub fn window(&self) -> &Window {
        if let Some(ref inner) = self.inner {
            &inner.window
        } else {
            unreachable!(
                "The context should be used only after executing initialize on a LumaSpace"
            );
        }
    }
}

impl<T: LumaHandler> ApplicationHandler<LumaEvent> for LumaContext<T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{JsCast, UnwrapThrowExt};
            use winit::{platform::web::WindowAttributesExtWebSys, window::WindowAttributes};
            const CANVAS_ID: &str = "canvas";

            let window = vello::wgpu::web_sys::window().expect("Window not found");
            let document = window.document().expect("Document not found");
            let canvas = document
                .get_element_by_id(CANVAS_ID)
                .expect("Canvas with id 'canvas' not found");
            let html_canvas_element =
                canvas.unchecked_into::<vello::wgpu::web_sys::HtmlCanvasElement>();
            html_canvas_element.set_width(600);
            html_canvas_element.set_height(480);
            let window_attributes =
                WindowAttributes::default().with_canvas(Some(html_canvas_element));
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let proxy = self.proxy.clone().unwrap();
            wasm_bindgen_futures::spawn_local(async move {
                let backend = LumaBackend::new(window.clone())
                    .await
                    .expect("Failed to create backend");
                let ui = backend.create_ui().expect("Failed to create UI");
                proxy.send_event(LumaEvent::LumaContext(LumaContextInner {
                    ui,
                    window,
                    rendering_backend: backend,
                }));
                // Aqui você precisa devolver o backend para o seu estado (LumaContextInner)
                // Como você está dentro de um closure async, o ideal é enviar
                // um evento customizado para o EventLoop do Winit para popular o 'self.inner'
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let window_attributes = self.config.to_window_attribs();
            use std::sync::Arc;
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let backend = pollster::block_on(LumaBackend::new(window.clone())).unwrap();
            let ui = backend.create_ui().unwrap();
            self.inner = Some(LumaContextInner {
                ui,
                window,
                rendering_backend: backend,
            });
        }
        if let Some(ref mut inner) = self.inner {
            self.handler.on_event(
                LumaEvent::Created,
                &inner.window,
                &inner.ui,
                &mut inner.rendering_backend,
            );
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: LumaEvent) {
        match event {
            LumaEvent::LumaContext(ctx) => self.inner = Some(ctx),
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(ref mut inner) = self.inner {
            match event {
                winit::event::WindowEvent::RedrawRequested => self
                    .handler
                    .rerender(&inner.ui, &mut inner.rendering_backend),
                resto => self.handler.on_event(
                    LumaEvent::Window(resto),
                    &inner.window,
                    &inner.ui,
                    &mut inner.rendering_backend,
                ),
            }
        }
    }
}
