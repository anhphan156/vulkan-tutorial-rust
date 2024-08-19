pub fn check_validation_layer_support(
    entry: &ash::Entry,
    required_validation_layers: &[&'static str],
) -> bool {
    let layer_properties = unsafe {
        entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties")
    };

    if layer_properties.len() <= 0 {
        println!("No validation layer available");
        return false;
    }

    for required_layer_name in required_validation_layers.iter() {
        for layer_property in layer_properties.iter() {
            let test_layer_name = super::tools::vk_to_string(&layer_property.layer_name);
            if (*required_layer_name) == test_layer_name {
                return true;
            }
        }
    }

    false
}
