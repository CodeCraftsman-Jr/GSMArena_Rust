use gsmarena;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    let phone1_query = if args.len() > 1 {
        args[1].clone()
    } else {
        "iPhone 15 Pro".to_string()
    };

    let phone2_query = if args.len() > 2 {
        args[2].clone()
    } else {
        "Samsung Galaxy S24".to_string()
    };

    println!("Comparing:\n  1. {}\n  2. {}\n", phone1_query, phone2_query);
    
    // Fetch first phone
    println!("Fetching {}...", phone1_query);
    let search1 = gsmarena::search(&phone1_query).await?;
    if search1.is_empty() {
        println!("Phone '{}' not found!", phone1_query);
        return Ok(());
    }
    let phone1 = gsmarena::phone(&search1[0].url).await?;

    // Fetch second phone
    println!("Fetching {}...", phone2_query);
    let search2 = gsmarena::search(&phone2_query).await?;
    if search2.is_empty() {
        println!("Phone '{}' not found!", phone2_query);
        return Ok(());
    }
    let phone2 = gsmarena::phone(&search2[0].url).await?;

    println!("\n");
    println!("=".repeat(80));
    println!("PHONE COMPARISON");
    println!("=".repeat(80));
    println!();

    // Compare key specifications
    let specs_to_compare = vec![
        ("Display Size", "size"),
        ("Display Resolution", "resolution"),
        ("Chipset", "chipset"),
        ("CPU", "cpu"),
        ("Memory", "memory"),
        ("Storage", "internal"),
        ("Main Camera", "main camera"),
        ("Selfie Camera", "selfie camera"),
        ("Battery", "battery"),
        ("Charging", "charging"),
        ("OS", "os"),
        ("Price", "price"),
    ];

    println!("{:<25} | {:<30} | {:<30}", "Specification", phone1.name, phone2.name);
    println!("{}", "-".repeat(88));

    for (spec_label, spec_key) in specs_to_compare {
        let value1 = find_spec(&phone1, spec_key).unwrap_or("N/A".to_string());
        let value2 = find_spec(&phone2, spec_key).unwrap_or("N/A".to_string());
        
        println!("{:<25} | {:<30} | {:<30}", spec_label, value1, value2);
    }

    println!("\n");
    println!("Images:");
    if let Some(img1) = &phone1.img {
        println!("  {}: {}", phone1.name, img1);
    }
    if let Some(img2) = &phone2.img {
        println!("  {}: {}", phone2.name, img2);
    }

    Ok(())
}

fn find_spec(phone: &gsmarena::Phone, key: &str) -> Option<String> {
    let search_key = key.to_lowercase();
    phone.specs.iter()
        .find(|(k, _)| k.to_lowercase().contains(&search_key))
        .map(|(_, v)| v.clone())
}
