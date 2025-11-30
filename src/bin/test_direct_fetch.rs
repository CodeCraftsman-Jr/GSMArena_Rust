use reqwest::blocking;
use scraper::{Html, Selector};

fn main() {
    // Test fetching Acer phones page directly
    let url = "https://www.gsmarena.com/acer-phones-59.php";
    
    println!("Testing URL: {}\n", url);
    
    match blocking::get(url) {
        Ok(response) => {
            println!("Status: {}", response.status());
            
            match response.text() {
                Ok(body) => {
                    println!("Body length: {} bytes\n", body.len());
                    
                    let document = Html::parse_document(&body);
                    let phone_selector = Selector::parse("div.makers ul li a").unwrap();
                    
                    let phones: Vec<_> = document.select(&phone_selector).collect();
                    println!("Found {} phone elements", phones.len());
                    
                    for (i, elem) in phones.iter().take(5).enumerate() {
                        let href = elem.value().attr("href").unwrap_or("no href");
                        let name = elem.text().collect::<String>().trim().to_string();
                        println!("  {}. {} -> {}", i + 1, name, href);
                    }
                    
                    // Check if we're being blocked
                    if body.contains("Too Many Requests") || body.contains("403") || body.contains("blocked") {
                        println!("\n⚠ We are being rate limited/blocked!");
                    }
                }
                Err(e) => eprintln!("Error reading body: {}", e),
            }
        }
        Err(e) => eprintln!("Error making request: {}", e),
    }
}
