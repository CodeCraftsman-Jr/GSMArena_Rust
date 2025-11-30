use gsmarena;
use gsmarena_scraper::models::get_device_name;
use serde_json;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let phone_id = if args.len() > 1 {
        args[1].clone()
    } else {
        "apple_iphone_15_pro_max-12548".to_string()
    };

    println!("Fetching detailed specifications for: {}\n", phone_id);
    
    let device_spec = gsmarena::get_specification(&phone_id);
    let json_val = serde_json::to_value(&device_spec)?;
    let name = get_device_name(&device_spec);

    println!("=== {} ===\n", name);

    // Display all specifications organized by category
    if let Some(specs) = json_val.get("specification").and_then(|v| v.as_array()) {
        for category in specs {
            if let Some(title) = category.get("category_title").and_then(|v| v.as_str()) {
                println!("[{}]", title);
                println!("{}", "-".repeat(60));
            }
            
            if let Some(category_specs) = category.get("category_spec").and_then(|v| v.as_array()) {
                for spec_pair in category_specs {
                    if let Some(spec_array) = spec_pair.as_array() {
                        if spec_array.len() == 2 {
                            if let (Some(key), Some(value)) = 
                                (spec_array[0].as_str(), spec_array[1].as_str()) {
                                println!("{:30} : {}", key, value);
                            }
                        }
                    }
                }
            }
            println!();
        }
    }

    // Save to JSON
    let filename = format!("{}.json", phone_id);
    let json = serde_json::to_string_pretty(&device_spec)?;
    std::fs::write(&filename, json)?;
    println!("✓ Saved to '{}'", filename);

    Ok(())
}
