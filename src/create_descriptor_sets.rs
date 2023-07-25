use vulkanalia::prelude::v1_0::*;
use anyhow::Result;
use std::mem::size_of;

use crate::UniformBufferObject;

pub unsafe fn create_descriptor_sets(
    device: &Device,
    descriptor_set_layout: &vk::DescriptorSetLayout,
    descriptor_pool: &vk::DescriptorPool,
    pool_size: usize,
    uniform_buffers: &Vec<vk::Buffer>
) -> Result<Vec<vk::DescriptorSet>> {
    let layouts = vec![*descriptor_set_layout; pool_size];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(*descriptor_pool)
        .set_layouts(&layouts);
        
    let descriptor_sets = device.allocate_descriptor_sets(&info)?;

    for i in 0..pool_size {
        for i in 0..pool_size {
            let info = vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i])
                .offset(0)
                .range(size_of::<UniformBufferObject>() as u64);

            let buffer_info = &[info];
            let ubo_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            device.update_descriptor_sets(&[ubo_write], &[] as &[vk::CopyDescriptorSet]);
        }
    }

    Ok(descriptor_sets)
}

pub unsafe fn create_descriptor_pool(device: &Device, pool_size: u32) -> Result<vk::DescriptorPool> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(pool_size);

    let pool_sizes = &[ubo_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(pool_size);

    let descriptor_pool = device.create_descriptor_pool(&info, None)?;

    Ok(descriptor_pool)
}

pub unsafe fn create_descriptor_set_layout(
    device: &Device,
) -> Result<vk::DescriptorSetLayout> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        // .stage_flags(vk::ShaderStageFlags::VERTEX & vk::ShaderStageFlags::FRAGMENT)
        .stage_flags(vk::ShaderStageFlags::ALL);

    let bindings = &[ubo_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings);        
    
    let descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;

    Ok(descriptor_set_layout)
}