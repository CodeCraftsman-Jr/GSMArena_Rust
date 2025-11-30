use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use rand::seq::SliceRandom;
use reqwest::blocking::Client as ReqwestClient;
use reqwest::Proxy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyDocument {
    #[serde(rename = "$id")]
    pub id: String,
    pub proxy: String, // Full proxy URL like "socks5://host:port" or "http://host:port"
    #[serde(rename = "type")]
    pub proxy_type: String, // "http", "socks4", "socks5"
    pub response_time: f64,
    pub tested_at: String,
    pub status: String, // "active", "inactive", etc.
    #[serde(rename = "$createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "$updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub id: String,
    pub proxy_url: String,
    pub proxy_type: String,
    pub response_time: f64,
    pub status: String,
}

impl From<ProxyDocument> for ProxyConfig {
    fn from(doc: ProxyDocument) -> Self {
        Self {
            id: doc.id,
            proxy_url: doc.proxy,
            proxy_type: doc.proxy_type,
            response_time: doc.response_time,
            status: doc.status,
        }
    }
}

impl ProxyConfig {
    /// Get the proxy URL
    pub fn to_url(&self) -> String {
        self.proxy_url.clone()
    }
}

#[derive(Debug, Deserialize)]
struct AppwriteListResponse {
    documents: Vec<ProxyDocument>,
}

pub struct ProxyManager {
    proxies: Arc<Mutex<Vec<ProxyConfig>>>,
    current_index: Arc<Mutex<usize>>,
    project_id: String,
    api_key: String,
    database_id: String,
    collection_id: String,
}

impl ProxyManager {
    /// Create a new ProxyManager from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let project_id = std::env::var("APPWRITE_PROJECT_ID")?;
        let api_key = std::env::var("APPWRITE_API_KEY")?;
        let database_id = std::env::var("APPWRITE_DATABASE_ID")?;
        let collection_id = std::env::var("APPWRITE_COLLECTION_ID")?;

        Ok(Self::new(project_id, api_key, database_id, collection_id))
    }

    /// Create a new ProxyManager with explicit credentials
    pub fn new(
        project_id: String,
        api_key: String,
        database_id: String,
        collection_id: String,
    ) -> Self {
        Self {
            proxies: Arc::new(Mutex::new(Vec::new())),
            current_index: Arc::new(Mutex::new(0)),
            project_id,
            api_key,
            database_id,
            collection_id,
        }
    }

    /// Fetch proxies from Appwrite
    pub fn fetch_proxies(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let url = format!(
            "https://cloud.appwrite.io/v1/databases/{}/collections/{}/documents",
            self.database_id, self.collection_id
        );

        let client = ReqwestClient::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let response = client
            .get(&url)
            .header("X-Appwrite-Project", &self.project_id)
            .header("X-Appwrite-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to fetch proxies: {}", error_text).into());
        }

        let app_response: AppwriteListResponse = response.json()?;
        
        let mut proxies = Vec::new();
        
        for doc in app_response.documents {
            // Only add proxies with "working" or "active" status
            let status_lower = doc.status.to_lowercase();
            if status_lower == "working" || status_lower == "active" {
                proxies.push(ProxyConfig::from(doc));
            }
        }

        let count = proxies.len();
        
        // Shuffle proxies for random selection
        let mut rng = rand::thread_rng();
        proxies.shuffle(&mut rng);
        
        *self.proxies.lock().unwrap() = proxies;
        *self.current_index.lock().unwrap() = 0;

        println!("✓ Loaded {} active proxies from Appwrite", count);
        
        Ok(count)
    }

    /// Get the next proxy in rotation
    pub fn get_next_proxy(&self) -> Option<ProxyConfig> {
        let proxies = self.proxies.lock().unwrap();
        
        if proxies.is_empty() {
            return None;
        }

        let mut index = self.current_index.lock().unwrap();
        let proxy = proxies[*index].clone();
        
        *index = (*index + 1) % proxies.len();
        
        Some(proxy)
    }

    /// Get a random proxy
    pub fn get_random_proxy(&self) -> Option<ProxyConfig> {
        let proxies = self.proxies.lock().unwrap();
        
        if proxies.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        proxies.choose(&mut rng).cloned()
    }

    /// Get all proxies
    pub fn get_all_proxies(&self) -> Vec<ProxyConfig> {
        self.proxies.lock().unwrap().clone()
    }

    /// Get proxy count
    pub fn proxy_count(&self) -> usize {
        self.proxies.lock().unwrap().len()
    }

    /// Create a reqwest client with the next proxy
    pub fn create_client_with_next_proxy(&self) -> Result<ReqwestClient, Box<dyn std::error::Error>> {
        if let Some(proxy_config) = self.get_next_proxy() {
            self.create_client_with_proxy(&proxy_config)
        } else {
            // No proxy available, return client without proxy
            Ok(ReqwestClient::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(30))
                .danger_accept_invalid_certs(true) // Accept self-signed certificates from proxies
                .build()?)
        }
    }

    /// Create a reqwest client with a specific proxy
    pub fn create_client_with_proxy(&self, proxy_config: &ProxyConfig) -> Result<ReqwestClient, Box<dyn std::error::Error>> {
        let proxy_url = proxy_config.to_url();
        
        // Format proxy URL based on type
        let formatted_proxy = match proxy_config.proxy_type.to_lowercase().as_str() {
            "http" | "https" => {
                if proxy_url.starts_with("http://") || proxy_url.starts_with("https://") {
                    proxy_url
                } else {
                    format!("http://{}", proxy_url)
                }
            }
            "socks4" => {
                if proxy_url.starts_with("socks4://") {
                    proxy_url
                } else {
                    format!("socks4://{}", proxy_url)
                }
            }
            "socks5" => {
                if proxy_url.starts_with("socks5://") {
                    proxy_url
                } else {
                    format!("socks5://{}", proxy_url)
                }
            }
            _ => proxy_url,
        };
        
        let proxy = Proxy::all(&formatted_proxy)?;

        Ok(ReqwestClient::builder()
            .proxy(proxy)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(15)) // Shorter timeout for proxies
            .danger_accept_invalid_certs(true) // Accept self-signed certificates
            .build()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_manager() {
        dotenv::dotenv().ok();
        
        if let Ok(manager) = ProxyManager::from_env() {
            match manager.fetch_proxies() {
                Ok(count) => {
                    println!("Fetched {} proxies", count);
                    
                    if let Some(proxy) = manager.get_next_proxy() {
                        println!("Next proxy: {}", proxy.to_url());
                    }
                }
                Err(e) => eprintln!("Error fetching proxies: {}", e),
            }
        }
    }
}
