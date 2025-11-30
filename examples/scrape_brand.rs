use gsmarena_scraper::{fetch_all_brands, fetch_phones_by_brand};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Get brand name from command line or use default
    let args: Vec<String> = std::env::args().collect();
    let brand_name = if args.len() > 1 {
        args[1].clone()
    } else {
        "Apple".to_string()
    };

    println!("Searching for brand: {}\n", brand_name);

    // Fetch all brands
    let brands = fetch_all_brands()?;
    
    // Find matching brand
    let brand = brands.iter()
        .find(|b| b.name.to_lowercase().contains(&brand_name.to_lowercase()))
        .ok_or(format!("Brand '{}' not found", brand_name))?;

    println!("Found: {}", brand.name);
    println!("Device count: {}", brand.device_count);
    println!("Slug: {}\n", brand.slug);

    // Fetch all phones for this brand
    println!("Fetching all phones...");
    let phones = fetch_phones_by_brand(&brand.slug)?;

    println!("\n✓ Found {} phones\n", phones.len());
    println!("{:-<80}", "");

    // Display all phones
    for (index, phone) in phones.iter().enumerate() {
        println!("{:3}. {}", index + 1, phone.name);
        println!("     ID: {}", phone.phone_id);
        if let Some(img) = &phone.image_url {
            println!("     Image: {}", img);
        }
        println!();
    }

    // Save to JSON
    let filename = format!("{}_phones.json", brand.name.to_lowercase().replace(" ", "_"));
    let json = serde_json::to_string_pretty(&phones)?;
    std::fs::write(&filename, json)?;
    println!("✓ Saved to '{}'", filename);

    Ok(())
}
