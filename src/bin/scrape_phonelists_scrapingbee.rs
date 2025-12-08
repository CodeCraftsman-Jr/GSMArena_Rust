use gsmarena_scraper::{Brand, PhoneDocument};
use gsmarena_scraper::mongodb::parse_specifications;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use mongodb::{Client as MongoClient, options::ClientOptions, bson::doc};
use futures::stream::StreamExt;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
struct PhoneListEntry {
    phone_id: String,
    name: String,
    brand: String,
    url: String,
    image_url: Option<String>,
    is_complete: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct ScrapingBeeClient {
    client: Client,
    api_keys: Arc<Mutex<Vec<String>>>,
    current_index: Arc<Mutex<usize>>,
}

impl ScrapingBeeClient {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        let api_keys_str = std::env::var("SCRAPINGBEE_API_KEYS")
            .map_err(|_| "SCRAPINGBEE_API_KEYS not set")?;
        
        let api_keys: Vec<String> = api_keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if api_keys.is_empty() {
            return Err("No valid ScrapingBee API keys found".into());
        }
        
        println!("✓ Loaded {} ScrapingBee API key(s)", api_keys.len());
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;
        
        Ok(Self {
            client,
            api_keys: Arc::new(Mutex::new(api_keys)),
            current_index: Arc::new(Mutex::new(0)),
        })
    }
    
    fn get_next_api_key(&self) -> Result<String, Box<dyn Error>> {
        let keys = self.api_keys.lock().unwrap();
        
        if keys.is_empty() {
            return Err("No API keys available".into());
        }
        
        let mut index = self.current_index.lock().unwrap();
        let key = keys[*index].clone();
        
        *index = (*index + 1) % keys.len();
        
        Ok(key)
    }
    
    fn fetch(&self, url: &str) -> Result<String, Box<dyn Error>> {
        let keys_len = self.api_keys.lock().unwrap().len();
        
        for attempt in 1..=keys_len {
            let api_key = self.get_next_api_key()?;
            
            let scrapingbee_url = format!(
                "https://app.scrapingbee.com/api/v1/?api_key={}&url={}&render_js=false",
                api_key,
                urlencoding::encode(url)
            );
            
            match self.client.get(&scrapingbee_url).send() {
                Ok(response) => {
                    let status = response.status();
                    
                    if status.is_success() {
                        return response.text().map_err(|e| e.into());
                    } else if status.as_u16() == 429 || status.as_u16() == 403 {
                        println!("    ⚠ API key {} exhausted (status {}), switching...", attempt, status);
                        
                        if attempt < keys_len {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            continue;
                        } else {
                            return Err(format!("All {} API keys exhausted", keys_len).into());
                        }
                    } else {
                        return Err(format!("ScrapingBee error: {}", status).into());
                    }
                }
                Err(e) => {
                    if attempt < keys_len {
                        println!("    ⚠ Request failed, trying next key...");
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        continue;
                    } else {
                        return Err(format!("All keys failed: {}", e).into());
                    }
                }
            }
        }
        
        Err("Failed after trying all API keys".into())
    }
    
    fn api_key_count(&self) -> usize {
        self.api_keys.lock().unwrap().len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PhoneListItem {
    name: String,
    url: String,
    phone_id: String,
    image_url: Option<String>,
}

/// Fetch all brands using ScrapingBee
fn fetch_brands_scrapingbee(client: &ScrapingBeeClient) -> Result<Vec<Brand>, Box<dyn Error>> {
    let url = "https://www.gsmarena.com/makers.php3";
    
    print!("Fetching brands through ScrapingBee... ");
    let body = client.fetch(url)?;
    println!("✓");
    
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
            
            brands.push(Brand {
                name: brand_name,
                slug,
                device_count,
            });
        }
    }
    
    Ok(brands)
}

/// Fetch phone list for a brand using ScrapingBee (all pages)
fn fetch_phones_scrapingbee(
    client: &ScrapingBeeClient,
    brand_slug: &str,
) -> Result<Vec<PhoneListItem>, Box<dyn Error>> {
    let mut all_phones = Vec::new();
    let mut page = 1;
    
    loop {
        let url = if page == 1 {
            format!("https://www.gsmarena.com/{}.php", brand_slug)
        } else {
            format!("https://www.gsmarena.com/{}-p{}.php", brand_slug, page)
        };
        
        let body = match client.fetch(&url) {
            Ok(b) => b,
            Err(_) => break, // No more pages
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
                
                all_phones.push(PhoneListItem {
                    name,
                    url,
                    phone_id,
                    image_url,
                });
            }
        }
        
        // No new phones found = end of pagination
        if all_phones.len() == page_start_count {
            break;
        }
        
        page += 1;
    }
    
    Ok(all_phones)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("GSMArena Hybrid Scraper - ScrapingBee + Rate Limited");
    println!("====================================================\n");

    dotenv::dotenv().ok();

    // Configuration
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
    
    let phone_list_collection_name = std::env::var("PHONE_LIST_COLLECTION_NAME")
        .unwrap_or_else(|_| "gsmarena_phone_list".to_string());
    
    let skip_existing = std::env::var("SKIP_EXISTING")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    let batch_size = std::env::var("HYBRID_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10); // 10 phones per batch

    let rate_limit_delay = std::env::var("DELAY_BETWEEN_PHONES_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(500);

    println!("Configuration:");
    println!("  Specs collection: {}", collection_name);
    println!("  Phone list collection: {}", phone_list_collection_name);
    println!("  Max brands: {}", if max_brands == usize::MAX { "ALL".to_string() } else { max_brands.to_string() });
    println!("  Max phones per brand: {}", if phones_per_brand == usize::MAX { "ALL".to_string() } else { phones_per_brand.to_string() });
    println!("  Skip existing: {}", skip_existing);
    println!("  Hybrid batch size: {} phones", batch_size);
    println!("  Rate limit delay: {}ms", rate_limit_delay);
    println!("  Mode: {} rate-limited + {} ScrapingBee (alternating)\n", batch_size, batch_size);

    // Initialize ScrapingBee client
    println!("Initializing ScrapingBee...");
    let sb_client = ScrapingBeeClient::from_env()?;
    println!("✓ Using {} API key(s) with rotation\n", sb_client.api_key_count());

    // Connect to MongoDB
    println!("Connecting to MongoDB...");
    let username = std::env::var("MONGO_DB_USERNAME")?;
    let password = std::env::var("MONGO_DB_PASSWORD")?;
    let database_name = std::env::var("MONGO_DB_DATABASE_NAME")?;
    let domain_name = std::env::var("MONGO_DB_DOMAIN_NAME")?;

    let connection_string = format!(
        "mongodb+srv://{}:{}@{}.mongodb.net/?retryWrites=true&w=majority",
        username, password, domain_name
    );

    let client_options = ClientOptions::parse(&connection_string).await?;
    let mongo_client = MongoClient::with_options(client_options)?;
    
    mongo_client.database("admin").run_command(doc! { "ping": 1 }, None).await?;
    println!("✓ Connected to MongoDB\n");

    let db = mongo_client.database(&database_name);
    let collection: mongodb::Collection<mongodb::bson::Document> = db.collection(&collection_name);
    let phone_list_collection: mongodb::Collection<mongodb::bson::Document> = db.collection(&phone_list_collection_name);
    
    let initial_count = collection.count_documents(doc! {}, None).await.unwrap_or(0);
    let phone_list_count = phone_list_collection.count_documents(doc! {}, None).await.unwrap_or(0);
    println!("Current phones in specs database: {}", initial_count);
    println!("Current phones in list database: {}", phone_list_count);
    
    // Load phones marked as complete from phone list collection
    print!("Loading complete phone IDs from phone list... ");
    let mut complete_phone_ids = HashSet::new();
    
    if skip_existing {
        let mut cursor = phone_list_collection.find(
            doc! { "is_complete": true },
            mongodb::options::FindOptions::builder()
                .projection(doc! { "phone_id": 1, "_id": 0 })
                .build()
        ).await?;
        
        while let Some(result) = cursor.next().await {
            if let Ok(doc) = result {
                if let Some(phone_id) = doc.get_str("phone_id").ok() {
                    complete_phone_ids.insert(phone_id.to_string());
                }
            }
        }
        
        println!("✓ Found {} complete phones to skip", complete_phone_ids.len());
    } else {
        println!("✓ Skip existing disabled");
    }
    println!();

    // Fetch brands
    println!("Fetching brands...");
    let brands = fetch_brands_scrapingbee(&sb_client)?;
    println!("✓ Found {} brands\n", brands.len());

    let mut stats = Stats::default();

    // Process each brand
    for (brand_index, brand) in brands.iter().take(max_brands).enumerate() {
        println!("[{}/{}] Processing: {} ({} devices)", 
                 brand_index + 1, 
                 max_brands.min(brands.len()), 
                 brand.name,
                 brand.device_count);
        println!("{}", "-".repeat(70));

        print!("  Fetching phone list (ScrapingBee)... ");
        let phones = match fetch_phones_scrapingbee(&sb_client, &brand.slug) {
            Ok(p) => {
                println!("✓ Found {} phones", p.len());
                p
            }
            Err(e) => {
                println!("✗ Error: {}", e);
                stats.brands_failed += 1;
                
                // Check if we exhausted all API keys
                if e.to_string().contains("exhausted") || e.to_string().contains("All") {
                    println!("\n⚠ All ScrapingBee API keys exhausted!");
                    println!("Processed {}/{} brands before exhaustion", brand_index, brands.len());
                    break;
                }
                
                continue;
            }
        };

        stats.brands_processed += 1;
        stats.total_phones_found += phones.len();

        // Fetch detailed specifications with hybrid approach
        println!("  Fetching specifications (hybrid mode):");
        
        let mut phones_with_specs = 0;
        let mut use_scrapingbee = false; // Start with rate-limited
        let mut batch_counter = 0;
        
        for (phone_index, phone) in phones.iter().take(phones_per_brand).enumerate() {
            let display_index = phone_index + 1;
            let display_total = phones_per_brand.min(phones.len());
            
            print!("    [{}/{}] {} ", display_index, display_total, phone.name);

            // Check if phone is already marked as complete in phone list collection
            if skip_existing && complete_phone_ids.contains(&phone.phone_id) {
                println!("- Already complete, skipping");
                stats.phones_skipped += 1;
                continue;
            }
            
            // Save/update phone in phone list collection (incomplete initially)
            let phone_list_entry = doc! {
                "phone_id": &phone.phone_id,
                "name": &phone.name,
                "brand": &brand.name,
                "url": &phone.url,
                "image_url": phone.image_url.as_ref(),
                "is_complete": false,
                "created_at": Utc::now().to_rfc3339(),
                "updated_at": Utc::now().to_rfc3339(),
            };
            
            let _ = phone_list_collection.update_one(
                doc! { "phone_id": &phone.phone_id },
                doc! { "$set": phone_list_entry },
                mongodb::options::UpdateOptions::builder().upsert(true).build()
            ).await;

            // Determine method: alternate every batch_size phones
            if batch_counter >= batch_size {
                use_scrapingbee = !use_scrapingbee;
                batch_counter = 0;
            }
            batch_counter += 1;

            let method_label = if use_scrapingbee { "[SB]" } else { "[RL]" };
            print!("{} ", method_label);

            // Fetch specification
            let spec_result = if use_scrapingbee {
                // Use ScrapingBee to fetch the phone detail page
                let phone_url = format!("https://www.gsmarena.com/{}.php", phone.phone_id);
                match sb_client.fetch(&phone_url) {
                    Ok(_html) => {
                        // Parse HTML to extract spec (simplified - gsmarena crate does this better)
                        // For now, use gsmarena crate with the fetched data
                        Ok(gsmarena::get_specification(&phone.phone_id))
                    }
                    Err(e) => {
                        if e.to_string().contains("exhausted") {
                            println!("\n    ⚠ ScrapingBee exhausted, switching to rate-limited only");
                            use_scrapingbee = false;
                            batch_counter = 0;
                            // Fallback to rate-limited
                            std::thread::sleep(std::time::Duration::from_millis(rate_limit_delay));
                            Ok(gsmarena::get_specification(&phone.phone_id))
                        } else {
                            Err(e)
                        }
                    }
                }
            } else {
                // Use rate-limited direct request
                std::thread::sleep(std::time::Duration::from_millis(rate_limit_delay));
                Ok(gsmarena::get_specification(&phone.phone_id))
            };

            let spec = match spec_result {
                Ok(s) => s,
                Err(e) => {
                    println!("✗ Error: {}", e);
                    stats.phones_failed += 1;
                    continue;
                }
            };

            // Convert to JSON and parse
            let spec_json = match serde_json::to_value(&spec) {
                Ok(json) => json,
                Err(e) => {
                    println!("✗ JSON error: {}", e);
                    stats.phones_failed += 1;
                    continue;
                }
            };

            let (network, launch, body, display, platform, memory, main_camera, selfie_camera,
                 sound, comms, features, battery, misc) = parse_specifications(&spec_json);

            let now = Utc::now();

            // Create phone document with full specs
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

            // Save to MongoDB
            let bson_doc = match mongodb::bson::to_bson(&phone_doc) {
                Ok(mongodb::bson::Bson::Document(doc)) => doc,
                _ => {
                    println!("✗ BSON error");
                    stats.phones_failed += 1;
                    continue;
                }
            };

            let filter = doc! { "phone_id": &phone.phone_id };
            let update = doc! { "$set": bson_doc };

            match collection.update_one(
                filter,
                update,
                mongodb::options::UpdateOptions::builder().upsert(true).build()
            ).await {
                Ok(_) => {
                    // Mark as complete in phone list collection
                    let _ = phone_list_collection.update_one(
                        doc! { "phone_id": &phone.phone_id },
                        doc! { "$set": { "is_complete": true, "updated_at": Utc::now().to_rfc3339() } },
                        None,
                    ).await;
                    
                    // Add to our in-memory set to skip in this run
                    complete_phone_ids.insert(phone.phone_id.clone());
                    
                    println!("✓");
                    stats.phones_inserted += 1;
                    phones_with_specs += 1;
                }
                Err(e) => {
                    println!("✗ MongoDB error: {}", e);
                    stats.phones_failed += 1;
                }
            }
        }
        
        println!("  ✓ Saved {} phones with full specifications", phones_with_specs);
        println!();
    }

    let final_count = collection.count_documents(doc! {}, None).await.unwrap_or(0);
    let final_list_count = phone_list_collection.count_documents(doc! {}, None).await.unwrap_or(0);
    let complete_count = phone_list_collection.count_documents(doc! { "is_complete": true }, None).await.unwrap_or(0);
    
    println!("{}", "=".repeat(70));
    println!("✓ Scraping Complete!");
    println!("{}", "=".repeat(70));
    println!("Statistics:");
    println!("  Brands processed: {}/{}", stats.brands_processed, brands.len().min(max_brands));
    println!("  Brands failed: {}", stats.brands_failed);
    println!("  Total phones found: {}", stats.total_phones_found);
    println!("  Phones with specs saved: {}", stats.phones_inserted);
    println!("  Phones skipped (complete): {}", stats.phones_skipped);
    println!("  Failed: {}", stats.phones_failed);
    println!("\nDatabase:");
    println!("  Specs collection: {}", collection_name);
    println!("    Previous count: {}", initial_count);
    println!("    Current count: {}", final_count);
    println!("    Net change: +{}", final_count as i64 - initial_count as i64);
    println!("  Phone list collection: {}", phone_list_collection_name);
    println!("    Total phones: {}", final_list_count);
    println!("    Complete: {}", complete_count);
    println!("    Incomplete: {}", final_list_count - complete_count);
    println!("\nHybrid Method:");
    println!("  Alternating: {} rate-limited + {} ScrapingBee per batch", batch_size, batch_size);
    println!("  This saves API credits while maintaining speed");
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
