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

    let size = (size_of::<Vertex>() * VERTICES.len()) as u64;
    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE
    )?;

    let memory = device.map_memory(
        vertex_buffer_memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());
    device.unmap_memory(vertex_buffer_memory);

    Ok((vertex_buffer, vertex_buffer_memory))
}