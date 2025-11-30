use gsmarena;
use gsmarena_scraper::models::get_device_name;
use serde_json;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Get phone IDs from command line or use defaults
    let args: Vec<String> = std::env::args().collect();
    
    let phone_ids = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec![
            "apple_iphone_15-12559".to_string(),
            "samsung_galaxy_s24-12771".to_string(),
            "google_pixel_8-12546".to_string(),
        ]
    };

    println!("Fetching specifications for {} phones\n", phone_ids.len());
    
    for (index, phone_id) in phone_ids.iter().enumerate() {
        println!("[{}/{}] Fetching: {}", index + 1, phone_ids.len(), phone_id);
        
        let spec = gsmarena::get_specification(phone_id);
        let name = get_device_name(&spec);
        let json_val = serde_json::to_value(&spec).unwrap();
        
        println!("✓ Success: {}", name);
        
        if let Some(specs) = json_val.get("specification").and_then(|v| v.as_array()) {
            println!("  Categories: {}", specs.len());
            
            if let Some(first_category) = specs.first() {
                if let Some(title) = first_category.get("category_title").and_then(|v| v.as_str()) {
                    println!("  First category: {}", title);
                }
            }
        }
        println!();
    }

    Ok(())
}
