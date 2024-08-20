use ash::vk;
use std::{ffi::CString, path::Path, ptr};

pub fn create_graphics_pipeline(device: &ash::Device) {
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

    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    };
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
