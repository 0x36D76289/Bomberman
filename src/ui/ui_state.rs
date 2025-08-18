use egui_winit_vulkano::Gui;

pub struct UiState {
    gui: Gui
}

impl UiState {
    pub fn is_transparent(&self) -> bool {
        false
    }
}
