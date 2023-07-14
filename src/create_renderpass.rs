use vulkanalia::prelude::v1_0::*;

use anyhow::Result;

pub trait RenderPassData {
    fn get_swapchain_format(&self) -> vk::Format;
    fn set_render_pass(&self, render_pass: vk::RenderPass);
}

pub unsafe fn create_render_pass(
    instance: &Instance,
    device: &Device,
    format: &vk::Format,
    out_render_pass: &mut vk::RenderPass
) -> Result<()> {
    
    let color_attachment = vk::AttachmentDescription::builder()
        .format(*format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments);

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let attachments = &[color_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);
    
    let render_pass = device.create_render_pass(&info, None)?;

    // TODO: Find alternative method of returning function, without overhead of creating unnecessary variable.
    // FIX: Check if gc could delete variable, since only pointer is passed.
    *out_render_pass = render_pass;

    Ok(())
}