use gsmarena::{DeviceSpecification};
use std::error::Error;

/// Wrapper around the gsmarena crate for easier usage
pub struct GsmArenaScraper;

impl GsmArenaScraper {
    /// Create a new GsmArenaScraper instance
    pub fn new() -> Self {
        Self
    }

    /// Get detailed specifications for a phone by its GSMArena ID
    /// Example ID: "apple_iphone_15_pro_max-12548"
    pub fn get_phone_details(&self, phone_id: &str) -> Result<DeviceSpecification, Box<dyn Error>> {
        let spec = gsmarena::get_specification(phone_id);
        Ok(spec)
    }

    /// Get phone specifications as JSON string
    pub fn get_phone_json(&self, phone_id: &str) -> Result<String, Box<dyn Error>> {
        let json = gsmarena::get_specification_json(phone_id);
        Ok(json)
    }

    /// Get multiple phone specifications
    pub fn get_multiple_phones(&self, phone_ids: &[&str]) -> Result<Vec<DeviceSpecification>, Box<dyn Error>> {
        let mut phones = Vec::new();
        
        for phone_id in phone_ids {
            match self.get_phone_details(phone_id) {
                Ok(phone) => phones.push(phone),
                Err(e) => eprintln!("Error fetching phone {}: {}", phone_id, e),
            }
        }

        Ok(phones)
    }
}

impl Default for GsmArenaScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_phone() {
        let scraper = GsmArenaScraper::new();
        let result = scraper.get_phone_details("apple_iphone_15-12559");
        assert!(result.is_ok());
    }
}
