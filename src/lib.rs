pub mod scraper;
pub mod models;
pub mod utils;
pub mod brand_scraper;

// Re-export main types
pub use scraper::GsmArenaScraper;
pub use gsmarena::{DeviceSpecification, Category, SingleSpecification};
pub use brand_scraper::{Brand, PhoneListItem, fetch_all_brands, fetch_phones_by_brand, fetch_all_phones};
