use gsmarena::DeviceSpecification;
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Save phone data to a JSON file
pub fn save_to_json<P: AsRef<Path>>(phone: &DeviceSpecification, path: P) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(phone)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Save multiple phones to a JSON file
pub fn save_phones_to_json<P: AsRef<Path>>(phones: &[DeviceSpecification], path: P) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(phones)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Format phone specifications as a readable string
pub fn format_phone_info(phone: &DeviceSpecification) -> String {
    let json_val = serde_json::to_value(phone).unwrap();
    let mut output = String::new();
    
    if let Some(name) = json_val.get("name").and_then(|v| v.as_str()) {
        output.push_str(&format!("Name: {}\n", name));
    }
    
    output.push_str("\nSpecifications:\n");
    
    if let Some(specs) = json_val.get("specification").and_then(|v| v.as_array()) {
        for category in specs {
            if let Some(title) = category.get("category_title").and_then(|v| v.as_str()) {
                output.push_str(&format!("\n[{}]\n", title));
            }
            
            if let Some(category_specs) = category.get("category_spec").and_then(|v| v.as_array()) {
                for spec_pair in category_specs {
                    if let Some(spec_array) = spec_pair.as_array() {
                        if spec_array.len() == 2 {
                            if let (Some(key), Some(value)) = 
                                (spec_array[0].as_str(), spec_array[1].as_str()) {
                                output.push_str(&format!("  {}: {}\n", key, value));
                            }
                        }
                    }
                }
            }
        }
    }

    output
}

/// Extract specific specification from phone data
pub fn extract_spec(phone: &DeviceSpecification, spec_name: &str) -> Option<String> {
    crate::models::find_spec_in_device(phone, spec_name)
}

/// Compare two phones by extracting key specifications
pub fn compare_phones(phone1: &DeviceSpecification, phone2: &DeviceSpecification) -> String {
    let mut output = String::new();
    
    let name1 = crate::models::get_device_name(phone1);
    let name2 = crate::models::get_device_name(phone2);
    
    output.push_str(&format!("Comparing: {} vs {}\n", name1, name2));
    output.push_str(&"=".repeat(50));
    output.push('\n');

    let specs_to_compare = vec![
        "Display",
        "Chipset",
        "Memory",
        "Battery",
        "Camera",
        "Price",
    ];

    for spec_name in specs_to_compare {
        let spec1 = extract_spec(phone1, spec_name).unwrap_or_else(|| "N/A".to_string());
        let spec2 = extract_spec(phone2, spec_name).unwrap_or_else(|| "N/A".to_string());
        
        output.push_str(&format!(
            "\n{}: \n  {}: {}\n  {}: {}\n",
            spec_name.to_uppercase(),
            name1,
            spec1,
            name2,
            spec2
        ));
    }

    output
}
