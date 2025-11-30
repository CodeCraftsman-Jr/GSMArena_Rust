use gsmarena_scraper::fetch_all_brands;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing brand fetcher...\n");

    let brands = fetch_all_brands()?;
    
    println!("✓ Found {} brands\n", brands.len());
    println!("First 20 brands:");
    println!("{:-<60}", "");
    
    for (i, brand) in brands.iter().take(20).enumerate() {
        println!("{:2}. {:30} | Devices: {:4} | Slug: {}", 
                 i + 1, brand.name, brand.device_count, brand.slug);
    }

    Ok(())
}
