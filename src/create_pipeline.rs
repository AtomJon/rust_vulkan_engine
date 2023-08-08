use vulkanalia::prelude::v1_0::*;
use anyhow::{Result, anyhow};

use crate::shader_manager::*;
use crate::vertex::*;

pub unsafe fn create_pipeline(device: &Device, shader_manager: &ShaderManager, swapchain_extent: &vk::Extent2D, descriptor_set_layout: &vk::DescriptorSetLayout, render_pass: &vk::RenderPass ) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
    let bytecode = shader_manager.get_shaders_bytecode()?;

    let vert_shader_module = create_shader_module(device, &bytecode.vertex)?;
    let frag_shader_module = create_shader_module(device, &bytecode.fragment)?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");
    
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    // let viewport = vk::Viewport::builder()
    //     .x(0.0)
    //     .y(0.0)
    //     .width(swapchain_extent.width as f32)
    //     .height(swapchain_extent.height as f32)
    //     .min_depth(0.0)
    //     .max_depth(1.0);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(swapchain_extent.height as f32)
        .width(swapchain_extent.width as f32)
        .height(-(swapchain_extent.height as f32))
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(*swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::_1);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::ONE)  // Optional
        .dst_color_blend_factor(vk::BlendFactor::ZERO) // Optional
        .color_blend_op(vk::BlendOp::ADD)              // Optional
        .src_alpha_blend_factor(vk::BlendFactor::ONE)  // Optional
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO) // Optional
        .alpha_blend_op(vk::BlendOp::ADD);             // Optional

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);


    let set_layouts = &[*descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts);

    let pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(*render_pass)
        .subpass(0);

    let pipeline = device.create_graphics_pipelines(
        vk::PipelineCache::null(), &[info], None)?.0;

    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);

    Ok((pipeline_layout, pipeline))
}

unsafe fn create_shader_module(
    device: &Device,
    bytecode: &[u8],
) -> Result<vk::ShaderModule> {
    let bytecode = Vec::<u8>::from(bytecode);
    let (prefix, code, suffix) = bytecode.align_to::<u32>();
    if !prefix.is_empty() || !suffix.is_empty() {
        return Err(anyhow!("Shader bytecode is not properly aligned."));
    }

    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.len())
        .code(code);

    Ok(device.create_shader_module(&info, None)?)
}