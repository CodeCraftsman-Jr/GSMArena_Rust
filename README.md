# GSMArena Rust Scraper

A Rust-based phone specification scraper using the [gsmarena](https://crates.io/crates/gsmarena) crate with additional web scraping capabilities to fetch all brands and phones. Now with **MongoDB integration** and **GitHub Actions** support for automated scraping!

## Features

- 📱 Fetch detailed phone specifications by GSMArena ID
- 🏢 Scrape all brands from GSMArena
- 📋 Get complete phone lists for any brand
- 🌍 Scrape entire database (all brands and phones)
- 💾 Save results to JSON files or MongoDB
- 🔄 **Automated scraping with GitHub Actions**
- 🗄️ **MongoDB integration for persistent storage**
- 🚀 Synchronous and asynchronous API
- 📊 Compare multiple phones
- 🎯 Type-safe with Rust

## Quick Links

- **[GitHub Actions Setup Guide](GITHUB_ACTIONS_SETUP.md)** - Set up automated scraping
- **[Quick Start Guide](QUICKSTART.md)** - Get started quickly

## Installation

1. Make sure you have Rust installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Build the project:
```bash
cd rust_scraper
cargo build --release
```

## Usage

### 🔄 Automated Scraping with GitHub Actions (Recommended)

Set up automated daily scraping to MongoDB:

1. **Follow the [GitHub Actions Setup Guide](GITHUB_ACTIONS_SETUP.md)**
2. Configure MongoDB credentials as GitHub Secrets
3. The workflow runs automatically daily at 2 AM UTC
4. Or trigger manually from the Actions tab

### 🗄️ Scrape to MongoDB

Run the MongoDB scraper locally:

```bash
# Copy and configure .env file
cp .env.example .env
# Edit .env with your MongoDB credentials

# Run the scraper (stores in MongoDB)
cargo run --bin scrape_to_mongodb

# Limit to specific brands/phones
cargo run --bin scrape_to_mongodb 5 10  # 5 brands, 10 phones each
```

### 1. Fetch All Brands and Their Phone Lists
```bash
cargo run --example fetch_all_brands
```
This fetches all brands and their phone lists (no detailed specs).
Output: `all_brands.json` and `all_phones_by_brand.json`

### 2. Scrape Complete Database (with detailed specs)
```bash
# Scrape 5 brands, 10 phones each (default for testing)
cargo run --example scrape_complete_database

# Custom: Scrape 20 brands, 50 phones each
cargo run --example scrape_complete_database 20 50

# Scrape ALL brands, ALL phones (WARNING: This takes HOURS!)
cargo run --example scrape_complete_database 999 999
```
Output: `scraped_data/` directory with organized JSON files

### 3. Scrape a Specific Brand
```bash
# Scrape all Apple phones
cargo run --example scrape_brand Apple

# Scrape Samsung phones
cargo run --example scrape_brand Samsung
```
Output: `[brand]_phones.json`

### 4. Get Detailed Phone Specifications
```bash
cargo run --example phone_specs apple_iphone_15_pro_max-12548
```

### 5. Compare Two Phones
```bash
cargo run --example compare_phones apple_iphone_15_pro-12559 samsung_galaxy_s24-12771
```

### 6. Fetch Multiple Phones
```bash
cargo run --example fetch_phones
```

### 7. Multi-Brand Comparison
```bash
cargo run --example multi_brand_comparison
```

## Finding Phone IDs

Phone IDs are extracted from GSMArena URLs. For example:
- URL: `https://www.gsmarena.com/apple_iphone_15_pro_max-12548.php`
- ID: `apple_iphone_15_pro_max-12548`

## Code Examples

### Fetch All Brands
```rust
use gsmarena_scraper::fetch_all_brands;

fn main() {
    let brands = fetch_all_brands().unwrap();
    for brand in brands {
        println!("{}: {} devices", brand.name, brand.device_count);
    }
}
```

### Fetch All Phones for a Brand
```rust
use gsmarena_scraper::{fetch_all_brands, fetch_phones_by_brand};

fn main() {
    let brands = fetch_all_brands().unwrap();
    let apple = brands.iter().find(|b| b.name == "Apple").unwrap();
    
    let phones = fetch_phones_by_brand(&apple.slug).unwrap();
    println!("Found {} Apple phones", phones.len());
}
```

### Get Phone Specifications
```rust
use gsmarena;

fn main() {
    let phone_id = "apple_iphone_15_pro_max-12548";
    let spec = gsmarena::get_specification(phone_id);
    println!("{}", gsmarena::get_specification_json(phone_id));
}
```

## Project Structure

```
rust_scraper/
├── Cargo.toml
├── README.md
├── QUICKSTART.md
├── src/
│   ├── main.rs              # Main entry point
│   ├── lib.rs               # Library root
│   ├── scraper.rs           # Scraper wrapper
│   ├── models.rs            # Data models
│   ├── utils.rs             # Utility functions
│   └── brand_scraper.rs     # Brand & phone list scraping
└── examples/
    ├── fetch_all_brands.rs          # Fetch all brands & phone lists
    ├── scrape_complete_database.rs  # Complete database scraper
    ├── scrape_brand.rs              # Scrape specific brand
    ├── fetch_phones.rs              # Fetch multiple phones
    ├── phone_specs.rs               # Get detailed specs
    ├── compare_phones.rs            # Compare two phones
    └── multi_brand_comparison.rs    # Multi-brand comparison
```

## Output Structure

### Complete Database Scraper Output
```
scraped_data/
├── all_brands.json              # List of all brands
├── scraping_stats.json          # Scraping statistics
├── Apple/
│   ├── phone_list.json          # All Apple phones
│   ├── all_specs.json           # Combined specifications
│   ├── apple_iphone_15-12559.json
│   └── apple_iphone_14-12345.json
├── Samsung/
│   ├── phone_list.json
│   ├── all_specs.json
│   └── ...
└── ...
```

## Dependencies

- `gsmarena` - Phone specification scraping
- `serde` / `serde_json` - JSON serialization
- `scraper` - HTML parsing
- `reqwest` - HTTP client (blocking)
- `regex` - Pattern matching
- `mongodb` - MongoDB driver for Rust
- `tokio` - Async runtime
- `dotenv` - Environment variable management
- `chrono` - Date/time handling

## MongoDB Integration

### Environment Variables

Create a `.env` file:

```bash
MONGO_DB_USERNAME=your_username
MONGO_DB_PASSWORD=your_password
MONGO_DB_DATABASE_NAME=your_database
MONGO_DB_DOMAIN_NAME=your_cluster_domain
COLLECTION_NAME=gsmarena_phones
MAX_BRANDS=5              # Optional: limit brands
PHONES_PER_BRAND=10       # Optional: limit phones per brand
SKIP_EXISTING=true        # Skip phones already in database
```

### Document Structure

Each phone is stored with:
- `phone_id`: Unique GSMArena ID
- `name`: Phone name
- `brand`: Manufacturer
- `url`: GSMArena URL
- `image_url`: Phone image
- `specifications`: Full spec JSON
- `scraped_at`: Timestamp

## GitHub Actions

The project includes automated scraping via GitHub Actions:

- **Schedule**: Runs daily at 2 AM UTC
- **Manual**: Trigger from Actions tab with custom parameters
- **Storage**: Results stored directly in MongoDB
- **Monitoring**: View logs and artifacts in Actions tab

See [GITHUB_ACTIONS_SETUP.md](GITHUB_ACTIONS_SETUP.md) for complete setup instructions.

## Performance Notes

- **Fetching all brands**: ~1-2 seconds
- **Fetching phone list for one brand**: ~1-5 seconds  
- **Fetching detailed specs for one phone**: ~1-2 seconds
- **Complete database scrape** (all brands, all phones): Several hours

The scraper includes delays between requests to be respectful to GSMArena's servers.

## Common Phone IDs

- `apple_iphone_15_pro_max-12548`
- `apple_iphone_15_pro-12559`
- `samsung_galaxy_s24_ultra-12771`
- `samsung_galaxy_s24-12771`
- `google_pixel_8_pro-12546`
- `oneplus_12-12712`
- `xiaomi_14_ultra-12764`

## License

MIT
