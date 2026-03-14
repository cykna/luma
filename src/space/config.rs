use winit::window::WindowAttributes;

///Luma configurations for the window that will be run
pub struct LumaWindowConfigs {
    ///The title of the window
    pub title: &'static str,
    ///Whether it should wait for new events to execute something new
    pub wait_for_events: bool,
    ///Whether the window should be decorated
    pub decorated: bool,
}
impl Default for LumaWindowConfigs {
    fn default() -> Self {
        Self {
            wait_for_events: false,
            title: "Luma",
            decorated: true,
        }
    }
}

impl LumaWindowConfigs {
    pub fn to_window_attribs(&self) -> WindowAttributes {
        WindowAttributes::default()
            .with_title(self.title)
            .with_transparent(true)
            .with_decorations(self.decorated)
            .with_active(true)
            .with_resizable(true)
    }
}
