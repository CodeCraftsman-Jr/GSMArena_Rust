use reqwest::blocking::Client;
use serde_json;

fn main() {
    dotenv::dotenv().ok();
    
    let project_id = std::env::var("APPWRITE_PROJECT_ID").expect("APPWRITE_PROJECT_ID not set");
    let api_key = std::env::var("APPWRITE_API_KEY").expect("APPWRITE_API_KEY not set");
    let database_id = std::env::var("APPWRITE_DATABASE_ID").expect("APPWRITE_DATABASE_ID not set");
    let collection_id = std::env::var("APPWRITE_COLLECTION_ID").expect("APPWRITE_COLLECTION_ID not set");
    
    println!("Testing Appwrite Connection");
    println!("============================");
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
        println!("Response Body (first 1000 chars):");
        println!("{}", &body[..body.len().min(1000)]);
        println!("\n\nFull JSON Response:");
        
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
            
            if let Some(docs) = json.get("documents").and_then(|d| d.as_array()) {
                println!("\n\n✓ Found {} documents", docs.len());
                
                if let Some(first_doc) = docs.first() {
                    println!("\nFirst document structure:");
                    println!("{}", serde_json::to_string_pretty(first_doc).unwrap());
                }
            } else {
                println!("\n⚠ No 'documents' array found in response");
            }
        }
    } else {
        let error_body = response.text().unwrap_or_else(|_| "Could not read error body".to_string());
        println!("Error Response:");
        println!("{}", error_body);
    }
}
