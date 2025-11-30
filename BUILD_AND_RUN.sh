# Build the project
cargo build --release

# Run the main example
cargo run --release

# Run specific examples:

# Search for phones
cargo run --example search "iPhone 15"

# Get detailed phone information
cargo run --example phone_details "Samsung Galaxy S24"

# Scrape phones by brand
cargo run --example brand_scraper "Apple"

# Compare two phones
cargo run --example compare "iPhone 15 Pro" "Samsung Galaxy S24 Ultra"

# Multi-brand comparison
cargo run --example multi_brand

# Run tests
cargo test

# Build for production
cargo build --release
