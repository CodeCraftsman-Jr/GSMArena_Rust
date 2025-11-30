use gsmarena_scraper::fetch_phones_by_brand;

fn main() {
    println!("Testing pagination for Acer (should be 113 devices)\n");
    
    match fetch_phones_by_brand("acer-phones-59") {
        Ok(phones) => {
            println!("✓ Found {} phones", phones.len());
            println!("\nFirst 10:");
            for (i, phone) in phones.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, phone.name);
            }
            println!("\nLast 10:");
            for (i, phone) in phones.iter().skip(phones.len().saturating_sub(10)).enumerate() {
                println!("  {}. {}", phones.len() - 9 + i, phone.name);
            }
        }
        Err(e) => eprintln!("✗ Error: {}", e),
    }
    
    println!("\n\nTesting pagination for alcatel (should be 413 devices)\n");
    
    match fetch_phones_by_brand("alcatel-phones-5") {
        Ok(phones) => {
            println!("✓ Found {} phones", phones.len());
            println!("\nFirst 10:");
            for (i, phone) in phones.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, phone.name);
            }
            println!("\nLast 10:");
            for (i, phone) in phones.iter().skip(phones.len().saturating_sub(10)).enumerate() {
                println!("  {}. {}", phones.len() - 9 + i, phone.name);
            }
        }
        Err(e) => eprintln!("✗ Error: {}", e),
    }
}
