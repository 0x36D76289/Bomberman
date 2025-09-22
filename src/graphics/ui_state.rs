use glam::Vec4;
use vulkano::{
    command_buffer::{RenderingAttachmentInfo, RenderingInfo, SubpassContents},
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    format::ClearValue,
    pipeline::{Pipeline, PipelineBindPoint, graphics::viewport::Viewport},
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
};

use crate::{
    game::resources::{ResourceName, Resources},
    graphics::{GuiPush, Renderer, Vulkan},
    ui::{UiState, utils::GetRatio},
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
        let (pipeline, command_buffer, rcx) = match (
            self.gui_pipeline.as_ref(),
            self.command_buffer.as_mut(),
            self.rcx.as_ref(),
        ) {
            (Some(pipeline), Some(command_buffer), Some(rcx)) => {
                (pipeline.clone(), command_buffer, rcx)
            }
            (None, _, _) => panic!("Pipeline is not initialized"),
            (_, None, _) => panic!("Command buffer is not started"),
            (_, _, None) => panic!("Render context is not initialized"),
        };

        let window_size: [u32; 2] = rcx.swapchain.image_extent();

        let rendering_info = {
            let mut color_attachment =
                RenderingAttachmentInfo::image_view(rcx.images[image_index as usize].clone());
            color_attachment.store_op = AttachmentStoreOp::Store;
            color_attachment.load_op = AttachmentLoadOp::Load;
            if is_first {
                color_attachment.load_op = AttachmentLoadOp::Clear;
                color_attachment.clear_value = Some(ClearValue::Float([0.0, 0.0, 0.0, 0.0]));
            }
            RenderingInfo {
                color_attachments: vec![Some(color_attachment)],
                layer_count: 1,
                contents: SubpassContents::Inline,
                ..Default::default()
            }
        };

        command_buffer.begin_rendering(rendering_info).unwrap();

        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: [window_size[0] as f32, window_size[1] as f32],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let layout = &pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new_variable(
            vulkan.descriptor_set_allocator.clone(),
            layout.clone(),
            resources.textures.len() as u32,
            [
                WriteDescriptorSet::sampler(0, self.sampler.clone()),
                WriteDescriptorSet::image_view_array(1, 0, resources.textures.clone()),
            ],
            [],
        )
        .unwrap();

        command_buffer
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap();

        let canvases = state.canvases.iter();
        let button_canvaces = {
            let mut ret = Vec::new();
            for canvas in state
                .buttons
                .iter()
                .map(|b| b.generate_canvases(window_size.get_ratio()))
                .flatten()
                .flatten()
            {
                ret.push(canvas);
            }
            ret
        };

        for canvas in canvases.chain(button_canvaces.iter()) {
            // draw the canvas
            let vertex_buffer = canvas.into_vertex_buffer(vulkan.memory_allocator.clone());
            let vertex_buffer_len = vertex_buffer.len() as u32;
            let push_constant = canvas.push_constant();

            command_buffer
                .push_constants(pipeline.layout().clone(), 0, push_constant)
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer)
                .unwrap();

            unsafe {
                command_buffer.draw(vertex_buffer_len, 1, 0, 0).unwrap();
            }

            // draw the text
            match &canvas.text {
                None => (),
                Some(text) => {
                    let vertex_buffer = self.text_renderer.render_str(
                        text,
                        canvas.text_size.unwrap_or(1.0),
                        canvas.center,
                        vulkan.memory_allocator.clone(),
                        window_size.get_ratio(),
                    );

                    let push_constant = GuiPush {
                        color: canvas.text_color.unwrap_or(Vec4::ONE).into(),
                        tex_index: resources.textures_index[&ResourceName::FontAtlas],
                    };
                    let vertex_buffer_len = vertex_buffer.len() as u32;

                    command_buffer
                        .push_constants(pipeline.layout().clone(), 0, push_constant)
                        .unwrap()
                        .bind_vertex_buffers(0, vertex_buffer)
                        .unwrap();

                    unsafe {
                        command_buffer.draw(vertex_buffer_len, 1, 0, 0).unwrap();
                    }
                }
            }
        }

        command_buffer.end_rendering().unwrap();
    }
}
