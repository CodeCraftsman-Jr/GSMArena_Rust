use gsmarena;
use serde_json;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("GSMArena Rust Scraper");
    println!("====================\n");

    // Example: Get phone specifications by GSMArena ID
    // The ID is the part of the URL: https://www.gsmarena.com/apple_iphone_15_pro_max-12548.php
    // So the ID would be: "apple_iphone_15_pro_max-12548"
    
    let phone_id = "apple_iphone_15_pro_max-12548";
    println!("Fetching specifications for phone ID: {}\n", phone_id);

    let device_spec = gsmarena::get_specification(phone_id);

    // Convert to JSON to access fields (since they're private)
    let json_val = serde_json::to_value(&device_spec)?;
    
    println!("--- Phone Details ---");
    if let Some(name) = json_val.get("name").and_then(|v| v.as_str()) {
        println!("Name: {}\n", name);
    }

    // Display specifications by category
    println!("Specifications:");
    if let Some(specs) = json_val.get("specification").and_then(|v| v.as_array()) {
        for category in specs {
            if let Some(title) = category.get("category_title").and_then(|v| v.as_str()) {
                println!("\n[{}]", title);
            }
            
            if let Some(category_specs) = category.get("category_spec").and_then(|v| v.as_array()) {
                for spec_pair in category_specs {
                    if let Some(spec_array) = spec_pair.as_array() {
                        if spec_array.len() == 2 {
                            if let (Some(key), Some(value)) = 
                                (spec_array[0].as_str(), spec_array[1].as_str()) {
                                println!("  {}: {}", key, value);
                            }
                        }
                    }
                }
            }
        }
    }

    // Save to JSON file
    let json_output = serde_json::to_string_pretty(&device_spec)?;
    std::fs::write("phone_details.json", json_output)?;
    println!("\n✓ Phone details saved to 'phone_details.json'");

    // Also try the direct JSON method
    let json_from_lib = gsmarena::get_specification_json(phone_id);
    std::fs::write("phone_details_alt.json", json_from_lib)?;
    println!("✓ Alternative JSON saved to 'phone_details_alt.json'");

    Ok(())
}
