use gsmarena;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get search query from command line or use default
    let args: Vec<String> = std::env::args().collect();
    let query = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "Samsung Galaxy S24".to_string()
    };

    println!("Searching for: {}\n", query);
    
    let results = gsmarena::search(&query).await?;

    println!("Found {} results:\n", results.len());
    
    for (index, phone) in results.iter().enumerate() {
        println!("{}. {}", index + 1, phone.name);
        println!("   URL: {}", phone.url);
        if let Some(img) = &phone.img {
            println!("   Image: {}", img);
        }
        println!();
    }

    Ok(())
}
