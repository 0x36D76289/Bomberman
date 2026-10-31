use crate::graphics::{Vulkan, renderer::RenderContext};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderingAttachmentInfo, RenderingInfo,
        SubpassContents,
    },
    format::ClearValue,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::graphics::viewport::Viewport,
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
};

#[allow(unsafe_code)]
pub fn draw(
    command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    vertex_count: u32,
) {
    unsafe { command_buffer.draw(vertex_count, 1, 0, 0) }.unwrap();
}

#[allow(unsafe_code)]
pub fn draw_indexed(
    command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    index_count: u32,
) {
    unsafe { command_buffer.draw_indexed(index_count, 1, 0, 0, 0) }.unwrap();
}

pub fn vertex_buffer<T: BufferContents>(
    vulkan: &Vulkan,
    vertices: impl ExactSizeIterator<Item = T>,
) -> Subbuffer<[T]> {
    Buffer::from_iter(
        vulkan.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vertices,
    )
    .unwrap()
}

pub fn viewport(extent: [u32; 2]) -> Viewport {
    Viewport {
        offset: [0.0, 0.0],
        extent: [extent[0] as f32, extent[1] as f32],
        depth_range: 0.0..=1.0,
    }
}

pub fn present_rendering(rcx: &RenderContext, image_index: u32, is_first: bool) -> RenderingInfo {
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
}
