#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

const MAX_FRAMES_IN_FLIGHT: usize = 2;
    
use std::ptr::copy_nonoverlapping as memcpy;
use std::time::Instant;
use std::mem::size_of;
use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_void;

use anyhow::{anyhow, Result};

use buffers::common::create_buffer;
use nalgebra_glm::{DVec2, make_vec2, Vec2};
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::window as vk_window;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent, VirtualKeyCode, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use log::*;

mod vertex;
use crate::vertex::VERTICES;

mod uniform_buffer_object;
use crate::uniform_buffer_object::UniformBufferObject;

mod create_renderpass;
use crate::create_renderpass::*;

mod create_descriptor_sets;
use create_descriptor_sets::*;

mod queue_family_indices;
use queue_family_indices::*;

mod swapchain_support;
use swapchain_support::*;

mod create_swapchain;
use create_swapchain::*;

mod create_framebuffers;
use create_framebuffers::*;

mod buffers;
use buffers::create_vertex_buffer::*;

mod create_pipeline;
use create_pipeline::*;

mod shader_manager;
use shader_manager::*;

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Window

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App

    let mut app = unsafe { App::create(&window)? };
    let mut destroying = false;
    let mut minimized = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed nor minimized.
            Event::MainEventsCleared if !destroying && !minimized =>
                unsafe { app.render(&window) }.unwrap(),


            Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                debug!("This input event was recorded: {:#?}, the scancode is {}", input, input.scancode);

                if input.state == ElementState::Released {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => control_flow.set_exit(),
                        Some(VirtualKeyCode::R) => unsafe { app.reload_shader(&window) }.unwrap(),
                        _ => {}
                    }
                }
            },

            // Window is resized and swapchain needs to be recreated. If app is minimized, rendering will seize.
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } =>
            {
                if size.width == 0 && size.height == 0 {
                    minimized = true;
                } else {
                    minimized = false;
                    app.resized = true
                }
            }
            
            // Destroy our Vulkan app.
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe { app.device.device_wait_idle().unwrap(); }
                unsafe { app.destroy(); }
            }
            _ => {}
        }
    });
}

unsafe fn create_instance(
    window: &Window,
    entry: &Entry, 
    data: &mut AppData)
-> Result<Instance> {

    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    for ele in &extensions {
        debug!("Extension '{:?}' required for instance.", *ele);
    }

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));
    
    if VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?;

    if VALIDATION_ENABLED {
        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

/// Our Vulkan app.
#[derive(Clone, Debug)]
struct App {
    entry: Entry,
    instance: Instance,
    device: Device,
    data: AppData,
    frame: usize,
    start: Instant,
    resized: bool,

    shader_manager: ShaderManager
}

impl App {
    /// Creates our Vulkan app.
    unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;

        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;

        data.surface = vk_window::create_surface(&instance, window)?;

        pick_physical_device(&instance, &mut data)?;

        let device = create_logical_device(&instance, &mut data)?;

        let swapchain_data = create_swapchain(window, &instance, &device, &CreateSwapchainData { surface: data.surface, physical_device: data.physical_device })?;
        data.swapchain = swapchain_data.swapchain;
        data.swapchain_extent = swapchain_data.swapchain_extent;
        data.swapchain_format = swapchain_data.swapchain_format;
        data.swapchain_images = swapchain_data.swapchain_images;

        create_swapchain_image_views(&device, &mut data)?;
    
        create_render_pass(&instance, &device, &data.swapchain_format, &mut data.render_pass)?;
        data.descriptor_set_layout = create_descriptor_set_layout(&device)?;

        let shader_manager = ShaderManager::create()?;
        let (pipeline_layout, pipeline) = create_pipeline(&device, &shader_manager, &data.swapchain_extent, &data.descriptor_set_layout, &data.render_pass)?;
        data.pipeline_layout = pipeline_layout;
        data.pipeline = pipeline;
        
        create_framebuffers(&device, &data.swapchain_image_views, &data.render_pass, &data.swapchain_extent, &mut data.framebuffers)?;
        create_command_pool(&instance, &device, &mut data)?;
        create_uniform_buffers(&instance, &device, &mut data)?;

        let pool_size = data.swapchain_images.len();
        data.descriptor_pool = create_descriptor_pool(&device, pool_size as u32)?;
        data.descriptor_sets = create_descriptor_sets(
            &device,
            &data.descriptor_set_layout,
            &data.descriptor_pool,
            pool_size,
            &data.uniform_buffers
        )?;

        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(&instance, &device, &data.physical_device)?;
        data.vertex_buffer = vertex_buffer;
        data.vertex_buffer_memory = vertex_buffer_memory;

        create_command_buffers(&device, &mut data)?;

        create_sync_objects(&device, &mut data)?;

        Ok(Self { entry, instance, data, device, frame: 0, start: Instant::now(), resized: false, shader_manager })
    }

    /// Renders a frame for our Vulkan app.
    unsafe fn render(&mut self, window: &Window) -> Result<()> {
        self.device.wait_for_fences(
            &[self.data.in_flight_fences[self.frame]],
            true,
            u64::max_value(),
        )?;
    
        let result = self
            .device
            .acquire_next_image_khr(
                self.data.swapchain,
                u64::max_value(),
                self.data.image_available_semaphores[self.frame],
                vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, __)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };
        
        if !self.data.images_in_flight[image_index as usize].is_null() {
            self.device.wait_for_fences(
                &[self.data.images_in_flight[image_index as usize]],
                true,
                u64::max_value(),
            )?;
        }
    
        self.data.images_in_flight[image_index as usize] =
            self.data.in_flight_fences[self.frame];

        self.update_uniform_buffer(image_index)?;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index as usize]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[self.data.in_flight_fences[self.frame]])?;

        self.device.queue_submit(self.data.graphics_queue, &[submit_info], self.data.in_flight_fences[self.frame])?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.device.queue_present_khr(self.data.present_queue, &present_info);

        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        
        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    unsafe fn reload_shader(&mut self, window: &Window) -> Result<()> {
        info!("Reloading shader");
        self.recreate_swapchain(window)?;

        Ok(())
    }

    unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;

        self.destroy_swapchain();

        let swapchain_create_data = CreateSwapchainData{
            surface: self.data.surface, physical_device: self.data.physical_device
        };

        let swapchain_data = create_swapchain(window, &self.instance, &self.device, &swapchain_create_data)?;
        
        self.data.swapchain = swapchain_data.swapchain;
        self.data.swapchain_extent = swapchain_data.swapchain_extent;
        self.data.swapchain_format = swapchain_data.swapchain_format;
        self.data.swapchain_images = swapchain_data.swapchain_images;

        create_render_pass(&self.instance, &self.device, &self.data.swapchain_format, &mut self.data.render_pass)?;
        
        let (pipeline_layout, pipeline) = create_pipeline(&self.device, &self.shader_manager, &self.data.swapchain_extent, &self.data.descriptor_set_layout, &self.data.render_pass)?;
        self.data.pipeline_layout = pipeline_layout;
        self.data.pipeline = pipeline;

        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_framebuffers(&self.device, &self.data.swapchain_image_views, &self.data.render_pass, &self.data.swapchain_extent, &mut self.data.framebuffers)?;
        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;

        let pool_size = self.data.swapchain_images.len();
        self.data.descriptor_pool = create_descriptor_pool(&self.device, pool_size as u32)?;
        self.data.descriptor_sets = create_descriptor_sets(&self.device, &self.data.descriptor_set_layout, &self.data.descriptor_pool, pool_size, &self.data.uniform_buffers)?;
        create_command_buffers(&self.device, &mut self.data)?;

        self.data
            .images_in_flight
            .resize(self.data.swapchain_images.len(), vk::Fence::null());
        
        Ok(())
    }

    unsafe fn destroy_swapchain(&mut self) {
        self.device.free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        self.device.destroy_descriptor_pool(self.data.descriptor_pool, None);
        self.data.uniform_buffers_memory.iter().for_each(|m| self.device.free_memory(*m, None));
        self.data.uniform_buffers.iter().for_each(|b| self.device.destroy_buffer(*b, None));
        self.data.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data.swapchain_image_views.iter().for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }    

    // TODO: CHANGE TO PUSH CONSTANT
    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {

        let extent = self.data.swapchain_extent;
        let resolution = Vec2::new(extent.width as f32, extent.height as f32);

        let time = self.start.elapsed().as_secs_f32();
        let ubo = UniformBufferObject {
            time: time,

            width: extent.width as f32,
            height: extent.height as f32
            // resolution: resolution
        };

        let memory = self.device.map_memory(
            self.data.uniform_buffers_memory[image_index],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(&ubo, memory.cast(), 1);

        self.device.unmap_memory(self.data.uniform_buffers_memory[image_index]);

        Ok(())
    }

    /// Destroys our Vulkan app.
    unsafe fn destroy(&mut self) {
        self.destroy_swapchain();

        self.device.destroy_buffer(self.data.vertex_buffer, None);
        self.device.free_memory(self.data.vertex_buffer_memory, None);
    
        self.data.in_flight_fences.iter().for_each(|f| self.device.destroy_fence(*f, None));
        self.data.render_finished_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.image_available_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));  
        self.device.destroy_command_pool(self.data.command_pool, None);  
        self.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);
        self.device.destroy_device(None);

        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }

        self.instance.destroy_surface_khr(self.data.surface, None);
        self.instance.destroy_instance(None);
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
struct AppData {
    surface: vk::SurfaceKHR,
    messenger: vk::DebugUtilsMessengerEXT,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
}

unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    data.uniform_buffers.clear();
    data.uniform_buffers_memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            instance,
            device,
            &data.physical_device,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        data.uniform_buffers.push(uniform_buffer);
        data.uniform_buffers_memory.push(uniform_buffer_memory);
    }

    Ok(())
}

unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        data.image_available_semaphores.push(device.create_semaphore(&semaphore_info, None)?);
        data.render_finished_semaphores.push(device.create_semaphore(&semaphore_info, None)?);

        data.in_flight_fences.push(device.create_fence(&fence_info, None)?);
    }

    data.images_in_flight = data.swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok(())
}

unsafe fn create_command_buffers(device: &Device, data: &mut AppData) -> Result<()> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&allocate_info)?;

    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        let inheritance = vk::CommandBufferInheritanceInfo::builder();
    
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty()) // Optional.
            .inheritance_info(&inheritance);             // Optional.
    
        device.begin_command_buffer(*command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);
        
        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };
            
        let clear_values = &[color_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(
            *command_buffer, &info, vk::SubpassContents::INLINE);    

        device.cmd_bind_pipeline(
            *command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline);

        // device.cmd_draw(*command_buffer, 3, 1, 0, 0);

        device.cmd_bind_descriptor_sets(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.pipeline_layout,
            0,
            &[data.descriptor_sets[i]],
            &[],
        );

        let index_count = VERTICES.len() as u32;

        device.cmd_bind_vertex_buffers(*command_buffer, 0, &[data.vertex_buffer], &[0]);
        device.cmd_draw(*command_buffer, index_count, 1, 0, 0);
            
        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}

unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, &data.surface, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty()) // Optional.
        .queue_family_index(indices.graphics);

    data.command_pool = device.create_command_pool(&info, None)?;

    Ok(())
}

unsafe fn create_logical_device(
    instance: &Instance,
    data: &mut AppData,
) -> Result<Device> {
    let indices = QueueFamilyIndices::get(instance, &data.surface, data.physical_device)?;

    // HashSet cannot contain duplicates, thus if the indices are equal, there will only be built one queue_family.
    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        vec![]
    };
    
    let features = vk::PhysicalDeviceFeatures::builder();

    let extensions = DEVICE_EXTENSIONS
    .iter()
    .map(|n| n.as_ptr())
    .collect::<Vec<_>>();

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(data.physical_device, &info, None)?;

    data.graphics_queue = device.get_device_queue(indices.graphics, 0);
    data.present_queue = device.get_device_queue(indices.present, 0);

    return Ok(device);
}



unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, data, physical_device) {
            warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
        } else {
            info!("Selected physical device (`{}`).", properties.device_name);
            data.physical_device = physical_device;
            return Ok(());
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}

unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {

    let properties = instance
        .get_physical_device_properties(physical_device);

    // if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
    //     return Err(anyhow!(SuitabilityError("Only discrete GPUs are supported.")));
    // }

    let features = instance
        .get_physical_device_features(physical_device);

    if features.geometry_shader != vk::TRUE {
        return Err(anyhow!(SuitabilityError("Missing geometry shader support.")));
    }

    QueueFamilyIndices::get(instance, &data.surface, physical_device)?;

    check_physical_device_extensions(instance, physical_device)?;

    let support = SwapchainSupport::get(instance, &data.surface, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }
    
    Ok(())    
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError("Missing required device extensions.")))
    }
}

unsafe fn create_swapchain_image_views(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    data.swapchain_image_views = data
        .swapchain_images
        .iter()
        .map(|i| {
            let components = vk::ComponentMapping::builder()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY);

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let info = vk::ImageViewCreateInfo::builder()
                .image(*i)
                .view_type(vk::ImageViewType::_2D)
                .format(data.swapchain_format)
                .components(components)
                .subresource_range(subresource_range);
            
            device.create_image_view(&info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    return vk::FALSE;
}