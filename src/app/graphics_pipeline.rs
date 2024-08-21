use ash::vk::{self, PipelineCache};
use std::{ffi::CString, path::Path, ptr};

use crate::util::structures::GraphicsPipelineStuff;

pub fn create_graphics_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
) -> GraphicsPipelineStuff {
    let vert_code = read_shader(Path::new("shaders/spv/triangle.vert.spv"));
    let frag_code = read_shader(Path::new("shaders/spv/triangle.frag.spv"));

    let vert_shader_module = create_shader_module(device, &vert_code);
    let frag_shader_module = create_shader_module(device, &frag_code);

    let entry_point = CString::new("main").unwrap();
    let vert_shader_stage = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::VERTEX,
        module: vert_shader_module,
        p_name: entry_point.as_ptr(),
        ..Default::default()
    };
    let frag_shader_stage = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: frag_shader_module,
        p_name: entry_point.as_ptr(),
        ..Default::default()
    };
    let shader_stages = [vert_shader_stage, frag_shader_stage];

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr(),
        _marker: std::marker::PhantomData,
    };

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: ptr::null(),
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: ptr::null(),
        _marker: std::marker::PhantomData,
    };

    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: vk::FALSE,
        _marker: std::marker::PhantomData,
    };

    let viewport_info = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        scissor_count: 1,
        ..Default::default()
    };

    let rasterizer_info = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::CLOCKWISE,
        depth_bias_enable: vk::FALSE,
        depth_bias_constant_factor: 0.0_f32,
        depth_bias_clamp: 0.0_f32,
        depth_bias_slope_factor: 0.0_f32,
        line_width: 1.0_f32,
        _marker: std::marker::PhantomData,
    };

    let multi_sampling_info = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 1.0_f32,
        p_sample_mask: ptr::null(),
        alpha_to_one_enable: vk::FALSE,
        alpha_to_coverage_enable: vk::FALSE,
        _marker: std::marker::PhantomData,
    };

    let color_blending_att = vk::PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::RGBA,
        blend_enable: vk::FALSE,
        ..Default::default()
    };

    let color_blending_info = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        logic_op_enable: vk::FALSE,
        attachment_count: 1,
        p_attachments: &color_blending_att,
        ..Default::default()
    };

    let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        ..Default::default()
    };

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .expect("Failed to create pipeline layout")
    };

    let pipeline_info = vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: ptr::null(),
        stage_count: 2,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_info,
        p_input_assembly_state: &input_assembly_info,
        p_viewport_state: &viewport_info,
        p_rasterization_state: &rasterizer_info,
        p_multisample_state: &multi_sampling_info,
        p_depth_stencil_state: ptr::null(),
        p_color_blend_state: &color_blending_info,
        p_dynamic_state: &dynamic_state_info,
        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        ..Default::default()
    };

    let graphics_pipeline = unsafe {
        device
            .create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None)
            .expect("Failed to create grahpics pipeline")
            .first()
            .unwrap()
            .to_owned()
    };

    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    };

    GraphicsPipelineStuff {
        graphics_pipeline,
        pipeline_layout,
    }
}

pub fn creat_render_pass(device: &ash::Device, swapchain_format: vk::Format) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription {
        format: swapchain_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        flags: vk::AttachmentDescriptionFlags::empty(),
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let subpass = vk::SubpassDescription {
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        ..Default::default()
    };

    let render_pass_info = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        attachment_count: 1,
        p_attachments: &color_attachment,
        subpass_count: 1,
        p_subpasses: &subpass,
        ..Default::default()
    };

    unsafe {
        device
            .create_render_pass(&render_pass_info, None)
            .expect("Failed to create renderpass")
    }
}

fn create_shader_module(device: &ash::Device, code: &Vec<u8>) -> vk::ShaderModule {
    let create_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
        _marker: std::marker::PhantomData,
    };

    unsafe {
        device
            .create_shader_module(&create_info, None)
            .expect("Failed to create shader module")
    }
}

fn read_shader(path: &Path) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;

    let spv_file = File::open(path).expect(&format!("Failed to read spv file at {:?}", path));
    let bytes: Vec<u8> = spv_file.bytes().filter_map(|x| x.ok()).collect();

    bytes
}
