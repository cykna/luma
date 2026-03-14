use vello::{Scene, wgpu::Device};

use crate::Result;

pub struct LumaUI {
    scene: Scene,
}

impl LumaUI {
    pub fn new(_device: &Device) -> Result<Self> {
        Ok(Self {
            scene: Scene::new(),
        })
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }
}

impl std::fmt::Debug for LumaUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = f.debug_struct("LumaUi").finish()?;
        write!(f, "{out:?}")
    }
}
