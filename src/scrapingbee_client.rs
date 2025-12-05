use reqwest::blocking::Client;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct ScrapingBeeClient {
    client: Client,
    api_keys: Arc<Mutex<Vec<String>>>,
    current_index: Arc<Mutex<usize>>,
}

impl ScrapingBeeClient {
    /// Create a new ScrapingBee client from comma-separated API keys in env
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let api_keys_str = std::env::var("SCRAPINGBEE_API_KEYS")
            .map_err(|_| "SCRAPINGBEE_API_KEYS not set in environment")?;
        
        let api_keys: Vec<String> = api_keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if api_keys.is_empty() {
            return Err("No valid ScrapingBee API keys found".into());
        }
        
        println!("✓ Loaded {} ScrapingBee API key(s)", api_keys.len());
        
        Ok(Self::new(api_keys))
    }
    
    /// Create a new ScrapingBee client with multiple API keys
    pub fn new(api_keys: Vec<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            api_keys: Arc::new(Mutex::new(api_keys)),
            current_index: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Get the next API key in rotation
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
    
    /// Fetch a URL through ScrapingBee with automatic API key rotation
    pub fn fetch(&self, url: &str) -> Result<String, Box<dyn Error>> {
        let keys_len = self.api_keys.lock().unwrap().len();
        
        // Try all API keys before giving up
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
                        // API key exhausted or blocked, try next key
                        println!("  ⚠ API key {} exhausted/blocked (status {}), switching to next key...", 
                                 attempt, status);
                        
                        if attempt < keys_len {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            continue;
                        } else {
                            return Err(format!("All {} API keys exhausted", keys_len).into());
                        }
                    } else {
                        return Err(format!("ScrapingBee returned status: {}", status).into());
                    }
                }
                Err(e) => {
                    if attempt < keys_len {
                        println!("  ⚠ Request failed ({}), trying next API key...", e);
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        continue;
                    } else {
                        return Err(format!("All API keys failed: {}", e).into());
                    }
                }
            }
        }
        
        Err("Failed to fetch after trying all API keys".into())
    }
    
    /// Get the number of API keys loaded
    pub fn api_key_count(&self) -> usize {
        self.api_keys.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrapingbee_client() {
        dotenv::dotenv().ok();
        
        if let Ok(client) = ScrapingBeeClient::from_env() {
            println!("Loaded {} API keys", client.api_key_count());
            
            // Test fetching a simple page
            match client.fetch("https://www.gsmarena.com/makers.php3") {
                Ok(html) => println!("✓ Successfully fetched page ({} bytes)", html.len()),
                Err(e) => eprintln!("✗ Failed to fetch: {}", e),
            }
        } else {
            println!("SCRAPINGBEE_API_KEYS not set, skipping test");
        }
    }
}
