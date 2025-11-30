pub mod scraper;
pub mod models;
pub mod utils;
pub mod brand_scraper;
pub mod mongodb;
pub mod proxy_manager;

// Re-export main types
pub use scraper::GsmArenaScraper;
pub use gsmarena::{DeviceSpecification, Category, SingleSpecification};
pub use brand_scraper::{Brand, PhoneListItem, fetch_all_brands, fetch_phones_by_brand, fetch_phones_by_brand_paginated, fetch_all_phones};
pub use mongodb::{MongoDBClient, PhoneDocument, parse_specifications};
pub use proxy_manager::{ProxyManager, ProxyConfig};
