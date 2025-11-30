use mongodb::{Client, options::ClientOptions, bson::doc, Collection};
use serde::{Deserialize, Serialize};
use std::error::Error;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneDocument {
    pub phone_id: String,
    pub name: String,
    pub brand: String,
    pub url: String,
    pub image_url: Option<String>,
    pub specifications: serde_json::Value,
    pub scraped_at: DateTime<Utc>,
}

pub struct MongoDBClient {
    client: Client,
    database_name: String,
}

impl MongoDBClient {
    /// Create a new MongoDB client from environment variables
    pub async fn from_env() -> Result<Self, Box<dyn Error>> {
        let username = std::env::var("MONGO_DB_USERNAME")
            .map_err(|_| "MONGO_DB_USERNAME not set")?;
        let password = std::env::var("MONGO_DB_PASSWORD")
            .map_err(|_| "MONGO_DB_PASSWORD not set")?;
        let database_name = std::env::var("MONGO_DB_DATABASE_NAME")
            .map_err(|_| "MONGO_DB_DATABASE_NAME not set")?;
        let domain_name = std::env::var("MONGO_DB_DOMAIN_NAME")
            .map_err(|_| "MONGO_DB_DOMAIN_NAME not set")?;

        // Construct MongoDB connection string
        // Format: mongodb+srv://username:password@domain.mongodb.net/
        let connection_string = format!(
            "mongodb+srv://{}:{}@{}.mongodb.net/?retryWrites=true&w=majority",
            username, password, domain_name
        );

        Self::new(&connection_string, &database_name).await
    }

    /// Create a new MongoDB client with custom connection string
    pub async fn new(connection_string: &str, database_name: &str) -> Result<Self, Box<dyn Error>> {
        let client_options = ClientOptions::parse(connection_string).await?;
        let client = Client::with_options(client_options)?;

        // Ping the server to verify connection
        client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await?;

        println!("✓ Successfully connected to MongoDB");

        Ok(MongoDBClient {
            client,
            database_name: database_name.to_string(),
        })
    }

    /// Get a collection for phone data
    pub fn get_collection(&self, collection_name: &str) -> Collection<PhoneDocument> {
        self.client
            .database(&self.database_name)
            .collection::<PhoneDocument>(collection_name)
    }

    /// Insert a single phone document
    pub async fn insert_phone(
        &self,
        collection_name: &str,
        phone: PhoneDocument,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.get_collection(collection_name);
        collection.insert_one(phone, None).await?;
        Ok(())
    }

    /// Insert multiple phone documents
    pub async fn insert_phones(
        &self,
        collection_name: &str,
        phones: Vec<PhoneDocument>,
    ) -> Result<usize, Box<dyn Error>> {
        if phones.is_empty() {
            return Ok(0);
        }

        let collection = self.get_collection(collection_name);
        let result = collection.insert_many(phones, None).await?;
        Ok(result.inserted_ids.len())
    }

    /// Update or insert a phone document (upsert based on phone_id)
    pub async fn upsert_phone(
        &self,
        collection_name: &str,
        phone: PhoneDocument,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.get_collection(collection_name);
        
        let filter = doc! { "phone_id": &phone.phone_id };
        let update = doc! {
            "$set": mongodb::bson::to_bson(&phone)?
        };

        collection
            .update_one(filter, update, mongodb::options::UpdateOptions::builder().upsert(true).build())
            .await?;

        Ok(())
    }

    /// Check if a phone already exists in the collection
    pub async fn phone_exists(
        &self,
        collection_name: &str,
        phone_id: &str,
    ) -> Result<bool, Box<dyn Error>> {
        let collection = self.get_collection(collection_name);
        let filter = doc! { "phone_id": phone_id };
        let count = collection.count_documents(filter, None).await?;
        Ok(count > 0)
    }

    /// Get the total count of phones in the collection
    pub async fn get_phone_count(
        &self,
        collection_name: &str,
    ) -> Result<u64, Box<dyn Error>> {
        let collection = self.get_collection(collection_name);
        let count = collection.count_documents(doc! {}, None).await?;
        Ok(count)
    }

    /// Delete all phones in the collection (use with caution!)
    pub async fn clear_collection(
        &self,
        collection_name: &str,
    ) -> Result<u64, Box<dyn Error>> {
        let collection = self.get_collection(collection_name);
        let result = collection.delete_many(doc! {}, None).await?;
        Ok(result.deleted_count)
    }
}
