use std::sync::Arc;
use vulkano::{pipeline::GraphicsPipeline, render_pass::RenderPass};

use crate::graphics::Vulkan;

pub struct UiRenderSystem {
    pipeline: Option<>,
}

impl UiRenderSystem {
    pub fn create_pipeline(&mut self, vulkan: &Vulkan, render_pass: Arc<RenderPass>) {
        
    }
}