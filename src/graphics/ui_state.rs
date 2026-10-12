use crate::{
    game::resources::Resources,
    graphics::{
        Renderer, Vulkan,
        draw::{draw, present_rendering, vertex_buffer, viewport},
    },
    ui::ui::UiState,
};
use vulkano::{
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    pipeline::{Pipeline, PipelineBindPoint},
};

impl Renderer {
    pub fn render_ui(
        &mut self,
        vulkan: &Vulkan,
        resources: &Resources,
        state: &UiState,
        image_index: u32,
        is_first: bool,
    ) {
        let pipeline = self.gui_pipeline.clone().expect("gui pipeline");
        let rcx = self.rcx.as_ref().expect("render context");
        let command_buffer = self.command_buffer.as_mut().expect("command buffer");

        command_buffer
            .begin_rendering(present_rendering(rcx, image_index, is_first))
            .unwrap();

        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [viewport(rcx.swapchain.image_extent())]
                    .into_iter()
                    .collect(),
            )
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                DescriptorSet::new_variable(
                    vulkan.descriptor_set_allocator.clone(),
                    pipeline.layout().set_layouts()[0].clone(),
                    resources.textures.len() as u32,
                    [
                        WriteDescriptorSet::sampler(0, self.ui_sampler.clone()),
                        WriteDescriptorSet::image_view_array(1, 0, resources.textures.clone()),
                    ],
                    [],
                )
                .unwrap(),
            )
            .unwrap();

        if !state.verts.is_empty() {
            let buffer = vertex_buffer(vulkan, state.verts.iter().copied());
            let count = buffer.len() as u32;
            command_buffer.bind_vertex_buffers(0, buffer).unwrap();
            draw(command_buffer, count);
        }

        command_buffer.end_rendering().unwrap();
    }
}
