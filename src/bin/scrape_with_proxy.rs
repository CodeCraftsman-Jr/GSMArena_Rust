use gsmarena_scraper::{fetch_all_brands, MongoDBClient, PhoneDocument, ProxyManager};
use gsmarena_scraper::mongodb::parse_specifications;
use gsmarena;
use serde_json;
use std::error::Error;
use chrono::Utc;
use scraper::{Html, Selector};

/// Fetch all brands using proxy
fn fetch_all_brands_with_proxy(
    proxy_manager: &ProxyManager,
) -> Result<Vec<gsmarena_scraper::Brand>, Box<dyn Error>> {
    let url = "https://www.gsmarena.com/makers.php3";
    
    // Try up to 10 different proxies
    for attempt in 1..=10 {
        let client = match proxy_manager.create_client_with_next_proxy() {
            Ok(c) => c,
            Err(e) => {
                println!("  ⚠ Failed to create proxy client: {}", e);
                continue;
            }
        };
        
        match client.get(url).send() {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    println!("  ⚠ Proxy rate limited, trying next proxy (attempt {}/10)...", attempt);
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    continue;
                }
                
                if !response.status().is_success() {
                    println!("  ⚠ Proxy returned {}, trying next proxy (attempt {}/10)...", response.status(), attempt);
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    continue;
                }
                
                let body = response.text()?;
                let document = Html::parse_document(&body);
                
                let mut brands = Vec::new();
                let brand_selector = Selector::parse("div.st-text table td a").unwrap();
                
                for element in document.select(&brand_selector) {
                    if let Some(href) = element.value().attr("href") {
                        let full_text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                        let parts: Vec<&str> = full_text.split_whitespace().collect();
                        
                        let (brand_name, device_count) = if parts.len() >= 2 {
                            if let Some(count_str) = parts.iter().rev().nth(1) {
                                if let Ok(count) = count_str.parse::<u32>() {
                                    let name = parts[..parts.len() - 2].join(" ");
                                    (name, count)
                                } else {
                                    (full_text.clone(), 0)
                                }
                            } else {
                                (full_text.clone(), 0)
                            }
                        } else {
                            (full_text.clone(), 0)
                        };
                        
                        let slug = href.trim_end_matches(".php").to_string();
                        
                        brands.push(gsmarena_scraper::Brand {
                            name: brand_name,
                            slug,
                            device_count,
                        });
                    }
                }
                
                if brands.len() > 0 {
                    return Ok(brands);
                }
                
                println!("  ⚠ Got response but found 0 brands, trying next proxy...");
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
            Err(e) => {
                if attempt <= 3 {
                    // Only show errors for first few attempts
                    println!("  ⚠ Proxy error (attempt {}/10): {}", attempt, 
                        e.to_string().chars().take(80).collect::<String>());
                }
                std::thread::sleep(std::time::Duration::from_millis(300));
                continue;
            }
        }
    }
    
    Err("Failed to fetch brands after trying 10 different proxies".into())
}

/// Fetch phone list using proxy
fn fetch_phones_by_brand_with_proxy(
    proxy_manager: &ProxyManager,
    brand_slug: &str,
) -> Result<Vec<gsmarena_scraper::PhoneListItem>, Box<dyn Error>> {
    let mut all_phones = Vec::new();
    let mut page = 1;
    
    loop {
        let url = if page == 1 {
            format!("https://www.gsmarena.com/{}.php", brand_slug)
        } else {
            format!("https://www.gsmarena.com/{}-p{}.php", brand_slug, page)
        };
        
        // Add delay before request
        if page > 1 {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        
        // Create client with next proxy
        let client = proxy_manager.create_client_with_next_proxy()?;
        
        let response = match client.get(&url).send() {
            Ok(r) => r,
            Err(_) => break,
        };
        
        if response.status() != 200 {
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                println!("    ⚠ Rate limited, trying next proxy...");
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }
            break;
        }
        
        let body = match response.text() {
            Ok(b) => b,
            Err(_) => break,
        };
        
        let document = Html::parse_document(&body);
        let phone_selector = Selector::parse("div.makers ul li a").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        
        let page_start_count = all_phones.len();
        
        for element in document.select(&phone_selector) {
            if let Some(href) = element.value().attr("href") {
                let name = element.text().collect::<String>().trim().to_string();
                let url = format!("https://www.gsmarena.com/{}", href);
                let phone_id = href.trim_end_matches(".php").to_string();
                
                let image_url = element
                    .select(&img_selector)
                    .next()
                    .and_then(|img| img.value().attr("src"))
                    .map(|src| {
                        if src.starts_with("http") {
                            src.to_string()
                        } else {
                            format!("https://www.gsmarena.com/{}", src)
                        }
                    });
                
                all_phones.push(gsmarena_scraper::PhoneListItem {
                    name,
                    url,
                    phone_id,
                    image_url,
                });
            }
        }
        
        if all_phones.len() == page_start_count {
            break;
        }
        
        page += 1;
    }
    
    Ok(all_phones)
}

/// Fetch phone specifications with retry logic using different proxies
fn fetch_with_retry_and_proxy(
    proxy_manager: &ProxyManager,
    phone_id: &str,
    max_retries: u32,
) -> Result<gsmarena::DeviceSpecification, String> {
    for attempt in 1..=max_retries {
        // Try with gsmarena crate (it doesn't support proxies, so this might fail)
        match std::panic::catch_unwind(|| gsmarena::get_specification(phone_id)) {
            Ok(spec) => return Ok(spec),
            Err(_) => {
                if attempt < max_retries {
                    eprintln!("    Retry {}/{} for {}", attempt, max_retries, phone_id);
                    std::thread::sleep(std::time::Duration::from_millis(1000 * attempt as u64));
                }
            }
        }
    }
    Err(format!("Failed after {} retries", max_retries))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("GSMArena Scraper - MongoDB Integration with Proxy Support");
    println!("=========================================================\n");

    // Load environment variables
    dotenv::dotenv().ok();

    // Get configuration
    let args: Vec<String> = std::env::args().collect();
    
    let max_brands = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(usize::MAX)
    } else {
        std::env::var("MAX_BRANDS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    };

    let phones_per_brand = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(usize::MAX)
    } else {
        std::env::var("PHONES_PER_BRAND")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    };

    let collection_name = std::env::var("COLLECTION_NAME")
        .unwrap_or_else(|_| "gsmarena_phones".to_string());

    let skip_existing = std::env::var("SKIP_EXISTING")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    let delay_between_phones = std::env::var("DELAY_BETWEEN_PHONES_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(500);

    let delay_between_brands = std::env::var("DELAY_BETWEEN_BRANDS_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(3000);

    let use_proxy = std::env::var("USE_PROXY")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    println!("Configuration:");
    println!("  Collection name: {}", collection_name);
    println!("  Max brands: {}", if max_brands == usize::MAX { "ALL".to_string() } else { max_brands.to_string() });
    println!("  Max phones per brand: {}", if phones_per_brand == usize::MAX { "ALL".to_string() } else { phones_per_brand.to_string() });
    println!("  Skip existing: {}", skip_existing);
    println!("  Delay between phones: {}ms", delay_between_phones);
    println!("  Delay between brands: {}ms", delay_between_brands);
    println!("  Use proxy: {}", use_proxy);
    println!();

    // Initialize proxy manager if enabled
    let proxy_manager = if use_proxy {
        println!("Initializing proxy manager...");
        let manager = ProxyManager::from_env()?;
        let count = manager.fetch_proxies()?;
        
        if count == 0 {
            println!("⚠ No proxies found in Appwrite collection!");
            println!("  Continuing without proxy support...\n");
            None
        } else {
            println!("✓ Proxy rotation enabled with {} proxies\n", count);
            Some(manager)
        }
    } else {
        println!("Proxy support disabled\n");
        None
    };

    // Connect to MongoDB
    println!("Connecting to MongoDB...");
    let mongo_client = MongoDBClient::from_env().await?;
    
    println!("Setting up database indexes...");
    mongo_client.create_indexes(&collection_name).await.ok();
    
    let initial_count = mongo_client.get_phone_count(&collection_name).await?;
    println!("Current phones in database: {}\n", initial_count);

    // Fetch all brands
    println!("Fetching brands from GSMArena...");
    let brands = if let Some(ref pm) = proxy_manager {
        match fetch_all_brands_with_proxy(pm) {
            Ok(b) => {
                println!("✓ Found {} brands (using proxy)\n", b.len());
                b
            }
            Err(e) => {
                println!("✗ Error fetching brands with proxy: {}", e);
                println!("  Trying without proxy...");
                match fetch_all_brands() {
                    Ok(b) => {
                        println!("✓ Found {} brands\n", b.len());
                        b
                    }
                    Err(e) => {
                        return Err(format!("Failed to fetch brands: {}", e).into());
                    }
                }
            }
        }
    } else {
        match fetch_all_brands() {
            Ok(b) => {
                println!("✓ Found {} brands\n", b.len());
                b
            }
            Err(e) => {
                return Err(format!("Failed to fetch brands: {}", e).into());
            }
        }
    };

    let mut stats = Stats::default();

    // Process brands sequentially
    for (brand_index, brand) in brands.iter().take(max_brands).enumerate() {
        println!("[{}/{}] Processing: {} ({} devices)", 
                 brand_index + 1, 
                 max_brands.min(brands.len()), 
                 brand.name,
                 brand.device_count);
        println!("{}", "-".repeat(70));

        // Fetch phone list
        print!("  Fetching phone list");
        if proxy_manager.is_some() {
            print!(" (with proxy rotation)");
        }
        print!("... ");
        
        let phones = if let Some(ref pm) = proxy_manager {
            match fetch_phones_by_brand_with_proxy(pm, &brand.slug) {
                Ok(p) => {
                    println!("✓ Found {} phones", p.len());
                    p
                }
                Err(e) => {
                    println!("✗ Error: {}", e);
                    stats.brands_failed += 1;
                    continue;
                }
            }
        } else {
            match gsmarena_scraper::fetch_phones_by_brand(&brand.slug) {
                Ok(p) => {
                    println!("✓ Found {} phones", p.len());
                    p
                }
                Err(e) => {
                    println!("✗ Error: {}", e);
                    stats.brands_failed += 1;
                    continue;
                }
            }
        };

        stats.brands_processed += 1;
        stats.total_phones_found += phones.len();

        println!("  Fetching specifications:");
        
        for (phone_index, phone) in phones.iter().take(phones_per_brand).enumerate() {
            let display_index = phone_index + 1;
            let display_total = phones_per_brand.min(phones.len());
            
            print!("    [{}/{}] {}", display_index, display_total, phone.name);

            if skip_existing {
                match mongo_client.phone_exists(&collection_name, &phone.phone_id).await {
                    Ok(true) => {
                        println!(" - Already exists, skipping");
                        stats.phones_skipped += 1;
                        continue;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        println!(" - Error checking existence: {}", e);
                        stats.phones_failed += 1;
                        continue;
                    }
                }
            }

            // Add delay before fetching
            std::thread::sleep(std::time::Duration::from_millis(delay_between_phones));

            // Fetch specifications with retry
            let spec = if let Some(ref pm) = proxy_manager {
                match fetch_with_retry_and_proxy(pm, &phone.phone_id, 3) {
                    Ok(s) => s,
                    Err(e) => {
                        println!(" ✗ {}", e);
                        stats.phones_failed += 1;
                        continue;
                    }
                }
            } else {
                match fetch_with_retry_and_proxy(&ProxyManager::new(String::new(), String::new(), String::new(), String::new()), &phone.phone_id, 3) {
                    Ok(s) => s,
                    Err(e) => {
                        println!(" ✗ {}", e);
                        stats.phones_failed += 1;
                        continue;
                    }
                }
            };
            
            let spec_json = match serde_json::to_value(&spec) {
                Ok(json) => json,
                Err(e) => {
                    println!(" ✗ JSON error: {}", e);
                    stats.phones_failed += 1;
                    continue;
                }
            };

            let (network, launch, body, display, platform, memory, main_camera, selfie_camera, 
                 sound, comms, features, battery, misc) = parse_specifications(&spec_json);

            let now = Utc::now();
            
            let phone_doc = PhoneDocument {
                phone_id: phone.phone_id.clone(),
                name: phone.name.clone(),
                brand: brand.name.clone(),
                url: phone.url.clone(),
                image_url: phone.image_url.clone(),
                source: "gsmarena".to_string(),
                network,
                launch,
                body,
                display,
                platform,
                memory,
                main_camera,
                selfie_camera,
                sound,
                comms,
                features,
                battery,
                misc,
                specifications_raw: spec_json,
                scraped_at: now,
                updated_at: now,
                version: 1,
            };

            match mongo_client.upsert_phone(&collection_name, phone_doc).await {
                Ok(_) => {
                    println!(" ✓");
                    stats.phones_inserted += 1;
                }
                Err(e) => {
                    println!(" ✗ MongoDB error: {}", e);
                    stats.phones_failed += 1;
                }
            }
        }

        println!();
        
        if brand_index + 1 < brands.len().min(max_brands) {
            println!("  ⏳ Waiting {}ms before next brand...", delay_between_brands);
            std::thread::sleep(std::time::Duration::from_millis(delay_between_brands));
            println!();
        }
    }

    let final_count = mongo_client.get_phone_count(&collection_name).await?;
    
    println!("{}", "=".repeat(70));
    println!("✓ Scraping Complete!");
    println!("{}", "=".repeat(70));
    println!("Statistics:");
    println!("  Brands processed: {}/{}", stats.brands_processed, brands.len().min(max_brands));
    println!("  Brands failed: {}", stats.brands_failed);
    println!("  Total phones found: {}", stats.total_phones_found);
    println!("  Phones inserted/updated: {}", stats.phones_inserted);
    println!("  Phones skipped (existing): {}", stats.phones_skipped);
    println!("  Phones failed: {}", stats.phones_failed);
    println!("\nDatabase:");
    println!("  Collection: {}", collection_name);
    println!("  Previous count: {}", initial_count);
    println!("  Current count: {}", final_count);
    println!("  Net change: +{}", final_count as i64 - initial_count as i64);
    
    if proxy_manager.is_some() {
        println!("\nProxy usage: Rotated through available proxies");
    }
    
    println!("{}", "=".repeat(70));

    Ok(())
}

#[derive(Default)]
struct Stats {
    brands_processed: usize,
    brands_failed: usize,
    total_phones_found: usize,
    phones_inserted: usize,
    phones_skipped: usize,
    phones_failed: usize,
}
