use log::debug;
use vulkanalia::prelude::v1_0::*;
use anyhow::Result;
use std::ptr::copy_nonoverlapping as memcpy;
use std::mem::size_of;

use crate::vertex::*;
use crate::buffers::common::*;

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    debug!("Creating vertex buffer");

    let buffer_info = vk::BufferCreateInfo::builder()
        .size((size_of::<Vertex>() * VERTICES.len()) as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .flags(vk::BufferCreateFlags::empty());

    let vertex_buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(vertex_buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            physical_device,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            requirements,
        )?);
    
    let vertex_buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)?;

    let memory = device.map_memory(
        vertex_buffer_memory,
        0,
        buffer_info.size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());
    device.unmap_memory(vertex_buffer_memory);

    Ok((vertex_buffer, vertex_buffer_memory))
}