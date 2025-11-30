use serde_json;

/// Re-export types from gsmarena crate
pub use gsmarena::{DeviceSpecification, Category, SingleSpecification};

/// Helper function to extract specs from DeviceSpecification using JSON
/// This is needed because the struct fields are private
pub fn device_to_json_value(device: &DeviceSpecification) -> serde_json::Value {
    serde_json::to_value(device).unwrap()
}

/// Extract a spec value from a DeviceSpecification by searching through JSON
pub fn find_spec_in_device(device: &DeviceSpecification, key: &str) -> Option<String> {
    let json_val = device_to_json_value(device);
    
    if let Some(specs) = json_val.get("specification").and_then(|v| v.as_array()) {
        let search_key = key.to_lowercase();
        
        for category in specs {
            if let Some(category_specs) = category.get("category_spec").and_then(|v| v.as_array()) {
                for spec_pair in category_specs {
                    if let Some(spec_array) = spec_pair.as_array() {
                        if spec_array.len() == 2 {
                            if let (Some(spec_key), Some(spec_value)) = 
                                (spec_array[0].as_str(), spec_array[1].as_str()) {
                                if spec_key.to_lowercase().contains(&search_key) {
                                    return Some(spec_value.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Get the name from a DeviceSpecification
pub fn get_device_name(device: &DeviceSpecification) -> String {
    let json_val = device_to_json_value(device);
    json_val.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string()
}
