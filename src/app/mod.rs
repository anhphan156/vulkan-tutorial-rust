extern crate glfw;

use crate::util::constants::{VALIDATION, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::util::debug;
use ash::{vk, Entry};
use glfw::{ffi::glfwTerminate, Action, ClientApiHint, Key, WindowEvent, WindowHint};
use std::{ffi::CString, ptr};

pub struct App {
    app_window: AppWindow,
    _entry: ash::Entry,
    instance: ash::Instance,
}

struct AppWindow {
    window: Option<glfw::PWindow>,
    events: Option<glfw::GlfwReceiver<(f64, WindowEvent)>>,
    glfw: Option<glfw::Glfw>,
}

impl App {
    pub fn new() -> App {
        let app_window = App::init_window();

        let entry = unsafe { Entry::load() }.unwrap();
        let instance = App::create_instance(&entry, &app_window);

        App {
            _entry: entry,
            app_window,
            instance,
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
            self.instance.destroy_instance(None);
            glfwTerminate();
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        _ => {}
    }
}
