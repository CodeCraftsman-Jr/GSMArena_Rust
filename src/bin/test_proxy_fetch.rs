use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyDocument {
    #[serde(rename = "$id")]
    pub id: String,
    pub proxy: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub response_time: f64,
    pub tested_at: String,
    pub status: String,
    #[serde(rename = "$createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "$updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppwriteListResponse {
    documents: Vec<ProxyDocument>,
    total: Option<usize>,
}

fn main() {
    dotenv::dotenv().ok();
    
    let project_id = std::env::var("APPWRITE_PROJECT_ID").expect("APPWRITE_PROJECT_ID not set");
    let api_key = std::env::var("APPWRITE_API_KEY").expect("APPWRITE_API_KEY not set");
    let database_id = std::env::var("APPWRITE_DATABASE_ID").expect("APPWRITE_DATABASE_ID not set");
    let collection_id = std::env::var("APPWRITE_COLLECTION_ID").expect("APPWRITE_COLLECTION_ID not set");
    
    println!("Testing Appwrite Proxy Collection");
    println!("===================================");
    println!("Project ID: {}", project_id);
    println!("Database ID: {}", database_id);
    println!("Collection ID: {}\n", collection_id);
    
    let url = format!(
        "https://cloud.appwrite.io/v1/databases/{}/collections/{}/documents",
        database_id, collection_id
    );
    
    println!("Fetching from: {}\n", url);
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build client");

    let response = client
        .get(&url)
        .header("X-Appwrite-Project", &project_id)
        .header("X-Appwrite-Key", &api_key)
        .header("Content-Type", "application/json")
        .send()
        .expect("Failed to send request");

    println!("Response Status: {}\n", response.status());
    
    if response.status().is_success() {
        let body = response.text().expect("Failed to read response");
        
        match serde_json::from_str::<AppwriteListResponse>(&body) {
            Ok(app_response) => {
                println!("✓ Successfully parsed response");
                println!("Total documents in collection: {}\n", app_response.total.unwrap_or(0));
                
                for (i, doc) in app_response.documents.iter().enumerate() {
                    println!("Proxy #{}", i + 1);
                    println!("  ID: {}", doc.id);
                    println!("  Proxy: {}", doc.proxy);
                    println!("  Type: {}", doc.proxy_type);
                    println!("  Status: {}", doc.status);
                    println!("  Response Time: {}ms", doc.response_time);
                    println!("  Tested At: {}", doc.tested_at);
                    println!();
                }
                
                let active_count = app_response.documents.iter()
                    .filter(|d| d.status.to_lowercase() == "active")
                    .count();
                    
                println!("Summary:");
                println!("  Total proxies: {}", app_response.documents.len());
                println!("  Active proxies: {}", active_count);
                
                // Show status breakdown
                let mut status_counts = std::collections::HashMap::new();
                for doc in &app_response.documents {
                    *status_counts.entry(doc.status.clone()).or_insert(0) += 1;
                }
                
                println!("\nStatus breakdown:");
                for (status, count) in status_counts {
                    println!("  {}: {}", status, count);
                }
            }
            Err(e) => {
                println!("✗ Failed to parse response: {}", e);
                println!("\nRaw response (first 2000 chars):");
                println!("{}", &body[..body.len().min(2000)]);
            }
        }
    } else {
        let error_body = response.text().unwrap_or_else(|_| "Could not read error body".to_string());
        println!("✗ Error Response:");
        println!("{}", error_body);
    }
}
