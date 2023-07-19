use vulkanalia::prelude::v1_0::*;
use anyhow::Result;

pub unsafe fn create_descriptor_set_layout(
    device: &Device,
    out_descriptor_set_layout: &mut vk::DescriptorSetLayout,
) -> Result<()> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let bindings = &[ubo_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings);        
    
    let descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;
    *out_descriptor_set_layout = descriptor_set_layout;

    Ok(())
}