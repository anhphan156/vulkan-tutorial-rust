pub mod graphics_pipeline;
extern crate glfw;

use crate::util::constants::{
    DEVICE_EXTENSIONS, MAX_FRAMES_IN_FLIGHT, VALIDATION, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::util::structures::{
    AppWindow, GraphicsPipelineStuff, QueueFamilyIndices, SurfaceStuff, SwapChainStuff,
    SwapChainSupportDetails, SyncObjects,
};
use crate::util::{debug, tools};
use ash::vk::CommandBufferResetFlags;
use ash::{vk, Entry};
use core::panic;
use glfw::{Action, ClientApiHint, Key, WindowHint};
use std::collections::HashSet;
use std::u32;
use std::{ffi::CString, ptr};

pub struct App {
    app_window: AppWindow,
    _entry: ash::Entry,
    instance: ash::Instance,
    _physical_device: vk::PhysicalDevice,
    device: ash::Device,
    _graphic_queue: vk::Queue,
    _present_queue: vk::Queue,
    surface_stuff: SurfaceStuff,
    swapchain_stuff: SwapChainStuff,
    swapchain_imageviews: Vec<vk::ImageView>,
    graphics_pipeline_stuff: GraphicsPipelineStuff,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    sync_objects: SyncObjects,
    current_frame: usize,
}

impl App {
    pub fn new() -> App {
        let app_window = App::init_window();
        let window = &app_window.window;

        let entry = unsafe { Entry::load() }.unwrap();
        let instance = App::create_instance(&entry, &app_window);
        let surface_stuff = App::create_surface(&entry, &instance, &window);
        let physical_device = App::pick_physical_device(&instance, &surface_stuff);
        let (device, indices) =
            App::create_logical_device(&instance, &physical_device, &surface_stuff);

        let graphic_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };

        let queue_family = App::find_queue_family(&instance, &physical_device, &surface_stuff);

        let swapchain_stuff = App::create_swapchain(
            &instance,
            &surface_stuff,
            &physical_device,
            &device,
            &queue_family,
        );
        let swapchain_imageviews = App::create_image_view(&device, &swapchain_stuff);

        let render_pass =
            graphics_pipeline::creat_render_pass(&device, swapchain_stuff.swapchain_format.clone());
        let graphics_pipeline_stuff =
            graphics_pipeline::create_graphics_pipeline(&device, render_pass.clone());

        let framebuffers = App::create_frame_buffers(
            &device,
            &swapchain_imageviews,
            swapchain_stuff.swapchain_extent,
            render_pass,
        );

        let command_pool = App::create_command_pool(&device, &queue_family);
        let command_buffers = App::create_command_buffers(&device, command_pool);

        let sync_objects = App::create_sync_objects(&device);

        App {
            _entry: entry,
            app_window,
            instance,
            _physical_device: physical_device,
            device,
            _graphic_queue: graphic_queue,
            _present_queue: present_queue,
            surface_stuff,
            swapchain_stuff,
            swapchain_imageviews,
            graphics_pipeline_stuff,
            render_pass,
            framebuffers,
            command_pool,
            command_buffers,
            sync_objects,
            current_frame: 0,
        }
    }
    fn create_instance(entry: &ash::Entry, app_window: &AppWindow) -> ash::Instance {
        if VALIDATION.enabled
            && !debug::check_validation_layer_support(entry, &VALIDATION.required_validation_layers)
        {
            panic!("Validation layer requested, but not available");
        }

        let app_name = CString::new("Vulkan App").unwrap();
        let engine_name = CString::new("Vulkan App").unwrap();
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            p_engine_name: engine_name.as_ptr(),
            application_version: vk::make_api_version(1, 1, 0, 0),
            engine_version: vk::make_api_version(1, 1, 0, 0),
            api_version: vk::make_api_version(1, 1, 0, 0),
            ..Default::default()
        };

        // Get Extension names
        let glfw = &app_window.glfw;
        let extension_names = glfw.get_required_instance_extensions().unwrap();
        let cstr_ext_names: Vec<_> = extension_names
            .iter()
            .map(|x| CString::new(x.as_str()).unwrap())
            .collect();
        let mut pp_ext_names: Vec<_> = cstr_ext_names.iter().map(|x| x.as_ptr()).collect();
        pp_ext_names.push(ptr::null());

        // Get Layers names
        let cstr_layer_names: Vec<_> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|x| CString::new(*x).unwrap())
            .collect();
        let pp_layer_names: Vec<*const i8> = cstr_layer_names.iter().map(|x| x.as_ptr()).collect();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_extension_names: pp_ext_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_layer_names: if VALIDATION.enabled {
                pp_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count: if VALIDATION.enabled {
                VALIDATION.required_validation_layers.len()
            } else {
                0
            } as u32,
            ..Default::default()
        };

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        };

        instance
    }

    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        surface_stuff: &SurfaceStuff,
    ) -> (ash::Device, QueueFamilyIndices) {
        let indices = App::find_queue_family(instance, physical_device, surface_stuff);
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family);
        unique_queue_families.insert(indices.present_family);

        let mut queue_create_infos = vec![];
        let queue_priorities = [1.0_f32];
        for &queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family.unwrap(),
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: 1,
                ..Default::default()
            };
            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        // Get Layers names
        let cstr_layer_names: Vec<_> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|x| CString::new(*x).unwrap())
            .collect();
        let pp_layer_names: Vec<*const i8> = cstr_layer_names.iter().map(|x| x.as_ptr()).collect();

        // Get Extensions names
        let enabled_extension_names = [ash::khr::swapchain::NAME.as_ptr()];

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: if VALIDATION.enabled {
                VALIDATION.required_validation_layers.len()
            } else {
                0
            } as u32,
            pp_enabled_layer_names: if VALIDATION.enabled {
                pp_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_extension_count: enabled_extension_names.len() as u32,
            pp_enabled_extension_names: enabled_extension_names.as_ptr(),
            p_enabled_features: &physical_device_features,
            _marker: std::marker::PhantomData,
        };

        let device: ash::Device = unsafe {
            instance
                .create_device(*physical_device, &device_create_info, None)
                .expect("Failed to create logical device")
        };

        (device, indices)
    }

    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &glfw::Window,
    ) -> SurfaceStuff {
        let surface = Box::new(vk::SurfaceKHR::null());
        let p_surface = Box::into_raw(surface.clone());

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        match window.create_window_surface(instance.handle(), ptr::null(), p_surface) {
            vk::Result::SUCCESS => SurfaceStuff {
                surface: unsafe { *p_surface },
                surface_loader,
            },
            _ => panic!("Failed to create surface"),
        }
    }

    fn create_swapchain(
        instance: &ash::Instance,
        surface_stuff: &SurfaceStuff,
        physical_device: &vk::PhysicalDevice,
        device: &ash::Device,
        queue_family: &QueueFamilyIndices,
    ) -> SwapChainStuff {
        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    2,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let swapchain_support = App::query_swapchain_support(surface_stuff, physical_device);

        let surface_format: vk::SurfaceFormatKHR =
            App::choose_swap_surface_format(&swapchain_support.formats);
        let present_mode: vk::PresentModeKHR =
            App::choose_swap_present_mode(&swapchain_support.present_modes);
        let extent: vk::Extent2D = App::choose_swap_extent(&swapchain_support.capabilities);

        let mut image_count: u32 = swapchain_support.capabilities.min_image_count + 1;
        if swapchain_support.capabilities.max_image_count > 0
            && image_count > swapchain_support.capabilities.max_image_count
        {
            image_count = swapchain_support.capabilities.max_image_count;
        }

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface_stuff.surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            queue_family_index_count,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            _marker: std::marker::PhantomData,
        };

        let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain")
        };
        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get swapchain images")
        };

        SwapChainStuff {
            swapchain,
            swapchain_loader,
            swapchain_images,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
        }
    }

    fn create_image_view(
        device: &ash::Device,
        swapchain_stuff: &SwapChainStuff,
    ) -> Vec<vk::ImageView> {
        let mut swapchain_imageviews = vec![];

        for &image in swapchain_stuff.swapchain_images.iter() {
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: swapchain_stuff.swapchain_format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                _marker: std::marker::PhantomData,
            };
            let imageview = unsafe {
                device
                    .create_image_view(&create_info, None)
                    .expect("Failed to create image view")
            };
            swapchain_imageviews.push(imageview);
        }

        swapchain_imageviews
    }

    fn create_frame_buffers(
        device: &ash::Device,
        swapchain_imageviews: &Vec<vk::ImageView>,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> Vec<vk::Framebuffer> {
        swapchain_imageviews
            .iter()
            .map(|x| {
                let frame_buffer_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                    render_pass,
                    attachment_count: 1,
                    p_attachments: x,
                    width: swapchain_extent.width,
                    height: swapchain_extent.height,
                    layers: 1,
                    ..Default::default()
                };

                unsafe {
                    device
                        .create_framebuffer(&frame_buffer_info, None)
                        .expect("Failed to create frambuffer")
                }
            })
            .collect()
    }

    fn create_command_pool(
        device: &ash::Device,
        queue_family: &QueueFamilyIndices,
    ) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: queue_family.graphics_family.unwrap(),
            ..Default::default()
        };

        unsafe {
            device
                .create_command_pool(&pool_info, None)
                .expect("Failed to create command pool")
        }
    }

    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> Vec<vk::CommandBuffer> {
        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAX_FRAMES_IN_FLIGHT,
            ..Default::default()
        };

        unsafe {
            device
                .allocate_command_buffers(&alloc_info)
                .expect("Failed to allocate command buffers")
        }
    }

    fn create_sync_objects(device: &ash::Device) -> SyncObjects {
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        unsafe {
            let mut image_available_semaphores =
                vec![vk::Semaphore::null(); MAX_FRAMES_IN_FLIGHT as usize];
            let mut render_finished_semaphores =
                vec![vk::Semaphore::null(); MAX_FRAMES_IN_FLIGHT as usize];
            let mut in_flight_fences = vec![vk::Fence::null(); MAX_FRAMES_IN_FLIGHT as usize];

            for i in 0..MAX_FRAMES_IN_FLIGHT as usize {
                image_available_semaphores[i] = device
                    .create_semaphore(&semaphore_info, None)
                    .expect("Failed to create semaphore");
                render_finished_semaphores[i] = device
                    .create_semaphore(&semaphore_info, None)
                    .expect("Failed to create semaphore");
                in_flight_fences[i] = device
                    .create_fence(&fence_info, None)
                    .expect("Failed to create fence");
            }

            SyncObjects {
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
            }
        }
    }

    fn record_command_buffer(&self, command_buffer: vk::CommandBuffer, image_index: u32) {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            ..Default::default()
        };

        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Failed to begin recording command buffer")
        };

        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32],
            },
        };
        let renderpass_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[image_index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_stuff.swapchain_extent,
            },
            clear_value_count: 1,
            p_clear_values: &clear_color,
            ..Default::default()
        };

        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &renderpass_info,
                vk::SubpassContents::INLINE,
            );

            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline_stuff.graphics_pipeline,
            );
        };

        let viewport = vk::Viewport {
            x: 0.0_f32,
            y: 0.0_f32,
            width: self.swapchain_stuff.swapchain_extent.width as f32,
            height: self.swapchain_stuff.swapchain_extent.height as f32,
            min_depth: 0.0_f32,
            max_depth: 1.0_f32,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain_stuff.swapchain_extent,
        };

        unsafe {
            self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_render_pass(command_buffer);
            self.device
                .end_command_buffer(command_buffer)
                .expect("Failed to record command buffer");
        };
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface_stuff: &SurfaceStuff,
    ) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        if physical_devices.len() == 0 {
            panic!("No available physical devices");
        }

        let mut result = None;
        for &physical_device in physical_devices.iter() {
            if App::is_physical_device_suitable(instance, &physical_device, surface_stuff) {
                result = Some(physical_device);
            }
        }

        match result {
            None => panic!("Failed to find a suitable GPU"),
            Some(physical_device) => physical_device,
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        surface_stuff: &SurfaceStuff,
    ) -> bool {
        let indices = App::find_queue_family(instance, physical_device, surface_stuff);
        let extensions_supported = App::check_device_extension_support(instance, physical_device);
        let swapchain_adequate = if extensions_supported {
            let swapchain_support = App::query_swapchain_support(surface_stuff, physical_device);
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };

        indices.is_complete() && extensions_supported && swapchain_adequate
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> bool {
        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(*physical_device)
                .expect("Failed to enumerate device extensions")
        };

        let mut available_extensions_names = vec![];
        for extension in available_extensions.iter() {
            let name = tools::vk_to_string(&extension.extension_name);
            available_extensions_names.push(name);
        }

        let mut required_extensions_names = HashSet::new();
        for extension in DEVICE_EXTENSIONS.names.iter() {
            required_extensions_names.insert(extension.to_string());
        }

        for name in available_extensions_names.iter() {
            required_extensions_names.remove(name);
        }

        required_extensions_names.is_empty()
    }

    fn query_swapchain_support(
        surface_stuff: &SurfaceStuff,
        physical_device: &vk::PhysicalDevice,
    ) -> SwapChainSupportDetails {
        let surface_loader = surface_stuff.surface_loader.clone();
        let surface = surface_stuff.surface;

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(*physical_device, surface)
                .expect("Failed to get surface format")
        };
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(*physical_device, surface)
                .expect("Failed to get surface capabilities")
        };
        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(*physical_device, surface)
                .expect("Failed to get surface present modes")
        };

        SwapChainSupportDetails {
            formats,
            capabilities,
            present_modes,
        }
    }

    fn choose_swap_surface_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        for available_format in available_formats.iter() {
            if available_format.format == vk::Format::R8G8B8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        available_formats.first().unwrap().clone()
    }

    fn choose_swap_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for &mode in available_present_modes.iter() {
            if mode == vk::PresentModeKHR::MAILBOX {
                return mode;
            }
        }
        return vk::PresentModeKHR::FIFO;
    }

    fn choose_swap_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: u32::clamp(
                    WINDOW_WIDTH,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: u32::clamp(
                    WINDOW_HEIGHT,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        surface_stuff: &SurfaceStuff,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let present_support = unsafe {
                surface_stuff
                    .surface_loader
                    .get_physical_device_surface_support(
                        *physical_device,
                        index,
                        surface_stuff.surface,
                    )
            }
            .unwrap_or(false);

            if present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    fn draw_frame(&mut self) {
        unsafe {
            let _ = self.device.wait_for_fences(
                &[self.sync_objects.in_flight_fences[self.current_frame]],
                true,
                u64::max_value(),
            );

            let _ = self
                .device
                .reset_fences(&[self.sync_objects.in_flight_fences[self.current_frame]]);

            let Ok((image_index, _)) = self.swapchain_stuff.swapchain_loader.acquire_next_image(
                self.swapchain_stuff.swapchain,
                u64::max_value(),
                self.sync_objects.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            ) else {
                panic!("failed to acquire next images")
            };

            let _ = self.device.reset_command_buffer(
                self.command_buffers[self.current_frame],
                CommandBufferResetFlags::empty(),
            );

            self.record_command_buffer(
                self.command_buffers[self.current_frame].clone(),
                image_index,
            );

            let wait_semaphores =
                [self.sync_objects.image_available_semaphores[self.current_frame]];
            let signal_semaphores =
                [self.sync_objects.render_finished_semaphores[self.current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let submit_info = vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                wait_semaphore_count: 1,
                p_wait_semaphores: wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask: wait_stages.as_ptr(),
                command_buffer_count: 1,
                p_command_buffers: &self.command_buffers[self.current_frame],
                signal_semaphore_count: 1,
                p_signal_semaphores: signal_semaphores.as_ptr(),
                ..Default::default()
            };

            self.device
                .queue_submit(
                    self._graphic_queue,
                    &[submit_info],
                    self.sync_objects.in_flight_fences[self.current_frame],
                )
                .expect("Failed to submit draw command buffer");

            let swapchains = [self.swapchain_stuff.swapchain];
            let present_info = vk::PresentInfoKHR {
                s_type: vk::StructureType::PRESENT_INFO_KHR,
                wait_semaphore_count: 1,
                p_wait_semaphores: signal_semaphores.as_ptr(),
                swapchain_count: 1,
                p_swapchains: swapchains.as_ptr(),
                p_image_indices: &image_index,
                ..Default::default()
            };

            self.swapchain_stuff
                .swapchain_loader
                .queue_present(self._present_queue, &present_info)
                .expect("Failed to present");
        };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT as usize;
    }

    fn init_window() -> AppWindow {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));

        let (mut window, events) = glfw
            .create_window(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                "Hello this is window",
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create GLFW window.");

        window.set_key_polling(true);

        AppWindow {
            window,
            events,
            glfw,
        }
    }
    pub fn main_loop(&mut self) {
        let mut frame_count: f64 = 0.0_f64;
        while !self.app_window.window.should_close() {
            self.app_window.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&self.app_window.events) {
                handle_window_event(&mut self.app_window.window, event);
            }

            self.draw_frame();
            frame_count += 1.0_f64;
            let t = self.app_window.glfw.get_time();
            println!("{}", frame_count / t);
        }

        unsafe {
            let _ = self.device.device_wait_idle();
        };
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT as usize {
                self.device
                    .destroy_semaphore(self.sync_objects.render_finished_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.sync_objects.image_available_semaphores[i], None);
                self.device
                    .destroy_fence(self.sync_objects.in_flight_fences[i], None);
            }
            self.device.destroy_command_pool(self.command_pool, None);
            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device
                .destroy_pipeline(self.graphics_pipeline_stuff.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.graphics_pipeline_stuff.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for &imageview in &self.swapchain_imageviews {
                self.device.destroy_image_view(imageview, None);
            }
            self.swapchain_stuff
                .swapchain_loader
                .destroy_swapchain(self.swapchain_stuff.swapchain, None);
            self.surface_stuff
                .surface_loader
                .destroy_surface(self.surface_stuff.surface, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        _ => {}
    }
}
