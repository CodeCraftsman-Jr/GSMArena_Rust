use gsmarena;
use gsmarena_scraper::models::{get_device_name, find_spec_in_device};
use serde_json;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // List of popular phone IDs to scrape
    let phone_ids = vec![
        "apple_iphone_15_pro_max-12548",
        "samsung_galaxy_s24_ultra-12771",
        "google_pixel_8_pro-12546",
        "oneplus_12-12712",
        "xiaomi_14_ultra-12764",
    ];

    println!("Scraping {} phones from multiple brands\n", phone_ids.len());
    
    let mut all_phones = Vec::new();

    for (index, phone_id) in phone_ids.iter().enumerate() {
        println!("[{}/{}] Fetching: {}", index + 1, phone_ids.len(), phone_id);
        
        let device_spec = gsmarena::get_specification(phone_id);
        let name = get_device_name(&device_spec);
        println!("  ✓ {}", name);
        all_phones.push(device_spec);
    }

    println!("\n");
    println!("{}", "=".repeat(100));
    println!("MULTI-BRAND COMPARISON");
    println!("{}", "=".repeat(100));
    println!();

    // Print summary table
    println!("{:<30} | {:<15} | {:<15} | {:<20} | {:<15}", 
             "Phone", "Display", "Chipset", "Camera", "Battery");
    println!("{}", "-".repeat(100));

    for phone in &all_phones {
        let name = get_device_name(phone);
        let display = find_spec_in_device(phone, "size").unwrap_or("N/A".to_string());
        let chipset = find_spec_in_device(phone, "chipset").unwrap_or("N/A".to_string());
        let camera = find_spec_in_device(phone, "main camera").unwrap_or("N/A".to_string());
        let battery = find_spec_in_device(phone, "battery").unwrap_or("N/A".to_string());
        
        println!("{:<30} | {:<15} | {:<15} | {:<20} | {:<15}",
                 truncate(&name, 30),
                 truncate(&display, 15),
                 truncate(&chipset, 15),
                 truncate(&camera, 20),
                 truncate(&battery, 15));
    }

    // Save to JSON
    let json = serde_json::to_string_pretty(&all_phones)?;
    std::fs::write("multi_brand_comparison.json", json)?;
    println!("\n✓ Saved to 'multi_brand_comparison.json'");

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}
