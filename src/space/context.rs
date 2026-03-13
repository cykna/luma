use flume::Sender;
use winit::{application::ApplicationHandler, window::Window};

use crate::{
    space::{LumaEvent, LumaWindowConfigs},
    ui::LumaUI,
};

#[derive(Default)]
pub struct LumaContext<E> {
    ui: LumaUI,
    config: LumaWindowConfigs,
    window: Option<Window>,
    rendering_backend: LumaBackend,
    pub(crate) sender: Option<Sender<LumaEvent<E>>>,
}

impl<E> LumaContext<E> {
    pub fn sender(&self) -> &Sender<LumaEvent<E>> {
        if let Some(ref sender) = self.sender {
            sender
        } else {
            unreachable!(
                "The context should be used only after executing initialize on a LumaSpace"
            );
        }
    }
    pub fn window(&self) -> &Window {
        if let Some(ref window) = self.window {
            window
        } else {
            unreachable!(
                "The context should be used only after executing initialize on a LumaSpace"
            );
        }
    }
}

impl<E: 'static> ApplicationHandler<E> for LumaContext<E> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(self.config.to_window_attribs())
                .unwrap(),
        );
        let _ = self.sender().send(LumaEvent::Created);
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
