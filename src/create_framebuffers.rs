use vulkanalia::prelude::v1_0::*;
use anyhow::Result;

pub unsafe fn create_framebuffers(device: &Device, swapchain_image_views: &Vec<vk::ImageView>, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D, out_framebuffers: &mut Vec<vk::Framebuffer>) -> Result<()> {
    let framebuffers = swapchain_image_views
        .iter()
        .map(|i| {
            let attachments = &[*i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*render_pass)
                .attachments(attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    *out_framebuffers = framebuffers;

    Ok(())
}