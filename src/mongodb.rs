use mongodb::{Client, options::ClientOptions, bson::doc, Collection, IndexModel};
use mongodb::options::IndexOptions;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneDocument {
    pub phone_id: String,
    pub name: String,
    pub brand: String,
    pub url: String,
    pub image_url: Option<String>,
    pub source: String, // Data source: "gsmarena"
    
    // Organized specifications by category
    pub network: Option<NetworkSpecs>,
    pub launch: Option<LaunchSpecs>,
    pub body: Option<BodySpecs>,
    pub display: Option<DisplaySpecs>,
    pub platform: Option<PlatformSpecs>,
    pub memory: Option<MemorySpecs>,
    pub main_camera: Option<CameraSpecs>,
    pub selfie_camera: Option<CameraSpecs>,
    pub sound: Option<SoundSpecs>,
    pub comms: Option<CommsSpecs>,
    pub features: Option<FeaturesSpecs>,
    pub battery: Option<BatterySpecs>,
    pub misc: Option<MiscSpecs>,
    
    // Raw specifications JSON (backup)
    pub specifications_raw: serde_json::Value,
    
    // Metadata
    pub scraped_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSpecs {
    pub technology: Option<String>,
    pub bands_2g: Option<String>,
    pub bands_3g: Option<String>,
    pub bands_4g: Option<String>,
    pub bands_5g: Option<String>,
    pub speed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchSpecs {
    pub announced: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodySpecs {
    pub dimensions: Option<String>,
    pub weight: Option<String>,
    pub build: Option<String>,
    pub sim: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySpecs {
    pub display_type: Option<String>,
    pub size: Option<String>,
    pub resolution: Option<String>,
    pub protection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSpecs {
    pub os: Option<String>,
    pub chipset: Option<String>,
    pub cpu: Option<String>,
    pub gpu: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpecs {
    pub card_slot: Option<String>,
    pub internal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSpecs {
    pub modules: Option<String>,
    pub features: Option<String>,
    pub video: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundSpecs {
    pub loudspeaker: Option<String>,
    pub jack_3_5mm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommsSpecs {
    pub wlan: Option<String>,
    pub bluetooth: Option<String>,
    pub positioning: Option<String>,
    pub nfc: Option<String>,
    pub radio: Option<String>,
    pub usb: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesSpecs {
    pub sensors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatterySpecs {
    pub battery_type: Option<String>,
    pub charging: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscSpecs {
    pub colors: Option<String>,
    pub models: Option<String>,
    pub sar: Option<String>,
    pub sar_eu: Option<String>,
    pub price: Option<String>,
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

    /// Create indexes for better query performance
    pub async fn create_indexes(
        &self,
        collection_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.get_collection(collection_name);
        
        // Index on phone_id (unique)
        let phone_id_index = IndexModel::builder()
            .keys(doc! { "phone_id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        
        // Index on brand
        let brand_index = IndexModel::builder()
            .keys(doc! { "brand": 1 })
            .build();
        
        // Index on scraped_at
        let scraped_index = IndexModel::builder()
            .keys(doc! { "scraped_at": -1 })
            .build();
        
        // Compound index on brand and name
        let brand_name_index = IndexModel::builder()
            .keys(doc! { "brand": 1, "name": 1 })
            .build();

        collection.create_indexes(vec![
            phone_id_index,
            brand_index,
            scraped_index,
            brand_name_index,
        ], None).await?;

        println!("✓ Created database indexes");
        Ok(())
    }
}

/// Helper function to parse specifications from raw JSON
pub fn parse_specifications(raw_specs: &serde_json::Value) -> (
    Option<NetworkSpecs>,
    Option<LaunchSpecs>,
    Option<BodySpecs>,
    Option<DisplaySpecs>,
    Option<PlatformSpecs>,
    Option<MemorySpecs>,
    Option<CameraSpecs>,
    Option<CameraSpecs>,
    Option<SoundSpecs>,
    Option<CommsSpecs>,
    Option<FeaturesSpecs>,
    Option<BatterySpecs>,
    Option<MiscSpecs>,
) {
    let specs_array = raw_specs.get("specification").and_then(|v| v.as_array());
    
    if specs_array.is_none() {
        return (None, None, None, None, None, None, None, None, None, None, None, None, None);
    }

    let mut specs_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    
    // Parse all specifications into a map
    for category in specs_array.unwrap() {
        let category_title = category.get("category_title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();
        
        let category_specs = category.get("category_spec")
            .and_then(|v| v.as_array());
        
        if let Some(specs) = category_specs {
            let mut category_map = HashMap::new();
            for spec_pair in specs {
                if let Some(spec_array) = spec_pair.as_array() {
                    if spec_array.len() == 2 {
                        if let (Some(key), Some(value)) = 
                            (spec_array[0].as_str(), spec_array[1].as_str()) {
                            category_map.insert(key.to_lowercase(), value.to_string());
                        }
                    }
                }
            }
            specs_map.insert(category_title, category_map);
        }
    }

    // Parse Network
    let network = if let Some(net) = specs_map.get("network") {
        Some(NetworkSpecs {
            technology: net.get("technology").cloned(),
            bands_2g: net.get("2g bands").cloned(),
            bands_3g: net.get("3g bands").cloned(),
            bands_4g: net.get("4g bands").cloned(),
            bands_5g: net.get("5g bands").cloned(),
            speed: net.get("speed").cloned(),
        })
    } else { None };

    // Parse Launch
    let launch = if let Some(lnch) = specs_map.get("launch") {
        Some(LaunchSpecs {
            announced: lnch.get("announced").cloned(),
            status: lnch.get("status").cloned(),
        })
    } else { None };

    // Parse Body
    let body = if let Some(bdy) = specs_map.get("body") {
        Some(BodySpecs {
            dimensions: bdy.get("dimensions").cloned(),
            weight: bdy.get("weight").cloned(),
            build: bdy.get("build").cloned(),
            sim: bdy.get("sim").cloned(),
        })
    } else { None };

    // Parse Display
    let display = if let Some(disp) = specs_map.get("display") {
        Some(DisplaySpecs {
            display_type: disp.get("type").cloned(),
            size: disp.get("size").cloned(),
            resolution: disp.get("resolution").cloned(),
            protection: disp.get("protection").cloned(),
        })
    } else { None };

    // Parse Platform
    let platform = if let Some(plat) = specs_map.get("platform") {
        Some(PlatformSpecs {
            os: plat.get("os").cloned(),
            chipset: plat.get("chipset").cloned(),
            cpu: plat.get("cpu").cloned(),
            gpu: plat.get("gpu").cloned(),
        })
    } else { None };

    // Parse Memory
    let memory = if let Some(mem) = specs_map.get("memory") {
        Some(MemorySpecs {
            card_slot: mem.get("card slot").cloned(),
            internal: mem.get("internal").cloned(),
        })
    } else { None };

    // Parse Main Camera
    let main_camera = if let Some(cam) = specs_map.get("main camera") {
        Some(CameraSpecs {
            modules: cam.get("single").or(cam.get("dual").or(cam.get("triple").or(cam.get("quad").or(cam.get("penta"))))).cloned(),
            features: cam.get("features").cloned(),
            video: cam.get("video").cloned(),
        })
    } else { None };

    // Parse Selfie Camera
    let selfie_camera = if let Some(cam) = specs_map.get("selfie camera") {
        Some(CameraSpecs {
            modules: cam.get("single").or(cam.get("dual")).cloned(),
            features: cam.get("features").cloned(),
            video: cam.get("video").cloned(),
        })
    } else { None };

    // Parse Sound
    let sound = if let Some(snd) = specs_map.get("sound") {
        Some(SoundSpecs {
            loudspeaker: snd.get("loudspeaker").cloned(),
            jack_3_5mm: snd.get("3.5mm jack").cloned(),
        })
    } else { None };

    // Parse Comms
    let comms = if let Some(com) = specs_map.get("comms") {
        Some(CommsSpecs {
            wlan: com.get("wlan").cloned(),
            bluetooth: com.get("bluetooth").cloned(),
            positioning: com.get("positioning").cloned(),
            nfc: com.get("nfc").cloned(),
            radio: com.get("radio").cloned(),
            usb: com.get("usb").cloned(),
        })
    } else { None };

    // Parse Features
    let features = if let Some(feat) = specs_map.get("features") {
        Some(FeaturesSpecs {
            sensors: feat.get("sensors").cloned(),
        })
    } else { None };

    // Parse Battery
    let battery = if let Some(bat) = specs_map.get("battery") {
        Some(BatterySpecs {
            battery_type: bat.get("type").cloned(),
            charging: bat.get("charging").cloned(),
        })
    } else { None };

    // Parse Misc
    let misc = if let Some(msc) = specs_map.get("misc") {
        Some(MiscSpecs {
            colors: msc.get("colors").cloned(),
            models: msc.get("models").cloned(),
            sar: msc.get("sar").cloned(),
            sar_eu: msc.get("sar eu").cloned(),
            price: msc.get("price").cloned(),
        })
    } else { None };

    (network, launch, body, display, platform, memory, main_camera, selfie_camera, sound, comms, features, battery, misc)
}
