use gsmarena;
use gsmarena_scraper::models::{get_device_name, find_spec_in_device};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    let phone1_id = if args.len() > 1 {
        args[1].clone()
    } else {
        "apple_iphone_15_pro-12559".to_string()
    };

    let phone2_id = if args.len() > 2 {
        args[2].clone()
    } else {
        "samsung_galaxy_s24-12771".to_string()
    };

    println!("Comparing:\n  1. {}\n  2. {}\n", phone1_id, phone2_id);
    
    // Fetch both phones
    println!("Fetching phone 1...");
    let phone1 = gsmarena::get_specification(&phone1_id);
    let name1 = get_device_name(&phone1);
    
    println!("Fetching phone 2...");
    let phone2 = gsmarena::get_specification(&phone2_id);
    let name2 = get_device_name(&phone2);

    println!("\n");
    println!("{}", "=".repeat(80));
    println!("PHONE COMPARISON");
    println!("{}", "=".repeat(80));
    println!();

    println!("{:<25} | {:<25} | {:<25}", "Specification", name1, name2);
    println!("{}", "-".repeat(80));

    // Compare key specifications
    let specs_to_compare = vec![
        ("Display size", "size"),
        ("Display Resolution", "resolution"),
        ("Chipset", "chipset"),
        ("CPU", "cpu"),
        ("Memory", "memory"),
        ("Internal", "internal"),
        ("Main camera", "main camera"),
        ("Selfie camera", "selfie camera"),
        ("Battery", "battery"),
        ("Charging", "charging"),
        ("OS", "os"),
        ("Price", "price"),
    ];

    for (spec_label, spec_key) in specs_to_compare {
        let value1 = find_spec_in_device(&phone1, spec_key).unwrap_or("N/A".to_string());
        let value2 = find_spec_in_device(&phone2, spec_key).unwrap_or("N/A".to_string());
        
        println!("{:<25} | {:<25} | {:<25}", spec_label, 
                 truncate(&value1, 25), truncate(&value2, 25));
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}
