use ash::vk;
use glfw::WindowEvent;

pub struct ValidationInfo {
    pub enabled: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub struct DeviceExtension {
    pub names: [&'static str; 1],
}

pub struct AppWindow {
    pub window: Option<glfw::PWindow>,
    pub events: Option<glfw::GlfwReceiver<(f64, WindowEvent)>>,
    pub glfw: Option<glfw::Glfw>,
}

pub struct SurfaceStuff {
    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}
impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct SwapChainStuff {
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_images: Vec<vk::Image>,
}

pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct GraphicsPipelineStuff {
    pub graphics_pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}
