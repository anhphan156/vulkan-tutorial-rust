pub mod graphics_pipeline;
extern crate glfw;

use crate::util::constants::{DEVICE_EXTENSIONS, VALIDATION, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::util::structures::{
    AppWindow, GraphicsPipelineStuff, QueueFamilyIndices, SurfaceStuff, SwapChainStuff,
    SwapChainSupportDetails,
};
use crate::util::{debug, tools};
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
}

impl App {
    pub fn new() -> App {
        let app_window = App::init_window();
        let window = app_window.window.as_ref().unwrap();

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
        let glfw = app_window.glfw.as_ref().unwrap();
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

    fn init_window() -> AppWindow {
        let mut app_window = AppWindow {
            window: None,
            events: None,
            glfw: None,
        };

        app_window.glfw = Some(glfw::init(glfw::fail_on_errors).unwrap());

        if let Some(ref mut glfw) = app_window.glfw {
            glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));

            let (w, e) = glfw
                .create_window(
                    WINDOW_WIDTH,
                    WINDOW_HEIGHT,
                    "Hello this is window",
                    glfw::WindowMode::Windowed,
                )
                .expect("Failed to create GLFW window.");

            app_window.window = Some(w);
            app_window.events = Some(e);

            if let Some(ref mut window) = app_window.window {
                window.set_key_polling(true);
            }
        }

        app_window
    }
    pub fn main_loop(&mut self) {
        let mut window = self.app_window.window.as_mut().unwrap();
        let events = self.app_window.events.as_ref().unwrap();
        while !window.should_close() {
            if let Some(ref mut glfw) = self.app_window.glfw {
                glfw.poll_events();
            }
            for (_, event) in glfw::flush_messages(&events) {
                handle_window_event(&mut window, event);
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
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
