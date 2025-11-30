use reqwest::blocking;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brand {
    pub name: String,
    pub slug: String,
    pub device_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneListItem {
    pub name: String,
    pub url: String,
    pub phone_id: String,
    pub image_url: Option<String>,
}

/// Fetch all brands from GSMArena
pub fn fetch_all_brands() -> Result<Vec<Brand>, Box<dyn Error>> {
    let url = "https://www.gsmarena.com/makers.php3";
    let response = blocking::get(url)?;
    let body = response.text()?;
    let document = Html::parse_document(&body);

    let mut brands = Vec::new();
    
    // Select brand links
    let brand_selector = Selector::parse("div.st-text table td a").unwrap();
    
    for element in document.select(&brand_selector) {
        if let Some(href) = element.value().attr("href") {
            // Get the full text (e.g., "Apple 123 devices")
            let full_text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
            
            // Try to extract device count and name
            let parts: Vec<&str> = full_text.split_whitespace().collect();
            
            let (brand_name, device_count) = if parts.len() >= 2 {
                // Check if second-to-last word is a number
                if let Some(count_str) = parts.iter().rev().nth(1) {
                    if let Ok(count) = count_str.parse::<u32>() {
                        // Everything before the count is the brand name
                        let name = parts[..parts.len() - 2].join(" ");
                        (name, count)
                    } else {
                        // No device count found, use full text as name
                        (full_text.clone(), 0)
                    }
                } else {
                    (full_text.clone(), 0)
                }
            } else {
                (full_text.clone(), 0)
            };
            
            // Extract slug from href (e.g., "apple-phones-48.php" -> "apple-phones-48")
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

/// Fetch all phones for a specific brand
pub fn fetch_phones_by_brand(brand_slug: &str) -> Result<Vec<PhoneListItem>, Box<dyn Error>> {
    fetch_phones_by_brand_paginated(brand_slug, usize::MAX)
}

/// Fetch phones for a specific brand with pagination support and max limit
pub fn fetch_phones_by_brand_paginated(brand_slug: &str, max_phones: usize) -> Result<Vec<PhoneListItem>, Box<dyn Error>> {
    let mut all_phones = Vec::new();
    let mut page = 1; // Start with page 1
    
    loop {
        if all_phones.len() >= max_phones {
            break;
        }
        
        // GSMArena pagination format:
        // Page 1: brand-phones-48.php
        // Page 2: brand-phones-48-p2.php  
        // Page 3: brand-phones-48-p3.php
        let url = if page == 1 {
            format!("https://www.gsmarena.com/{}.php", brand_slug)
        } else {
            format!("https://www.gsmarena.com/{}-p{}.php", brand_slug, page)
        };
        
        // Add delay before request to avoid rate limiting
        if page > 1 {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        
        let response = match blocking::get(&url) {
            Ok(r) => r,
            Err(_) => break,
        };
        
        if response.status() != 200 {
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
            if all_phones.len() >= max_phones {
                break;
            }
            
            if let Some(href) = element.value().attr("href") {
                let name = element.text().collect::<String>().trim().to_string();
                let url = format!("https://www.gsmarena.com/{}", href);
                
                // Extract phone ID from URL (e.g., "apple_iphone_15-12559.php" -> "apple_iphone_15-12559")
                let phone_id = href.trim_end_matches(".php").to_string();
                
                // Try to get image URL
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
        
        // If no new phones found on this page, we've reached the end
        if all_phones.len() == page_start_count {
            break;
        }
        
        page += 1;
    }
    
    Ok(all_phones)
}

/// Fetch all phones from all brands
pub fn fetch_all_phones() -> Result<Vec<(Brand, Vec<PhoneListItem>)>, Box<dyn Error>> {
    let brands = fetch_all_brands()?;
    let mut all_data = Vec::new();
    
    for brand in brands {
        println!("Fetching phones for: {} ({} devices)", brand.name, brand.device_count);
        
        match fetch_phones_by_brand(&brand.slug) {
            Ok(phones) => {
                println!("  ✓ Found {} phones", phones.len());
                all_data.push((brand, phones));
            }
            Err(e) => {
                eprintln!("  ✗ Error: {}", e);
            }
        }
        
        // Delay between brands
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    Ok(all_data)
}
