# Quick Start Guide - GSMArena Rust Scraper

## Installation

```powershell
cd rust_scraper
cargo build --release
```

## Basic Usage

### 1. Fetch Phone Details (Main Example)
```powershell
cargo run --release
```
This fetches details for iPhone 15 Pro Max and saves to JSON.

### 2. Compare Two Phones
```powershell
cargo run --release --example compare_phones apple_iphone_15-12559 samsung_galaxy_s24_ultra-12771
```

### 3. Get Detailed Specs
```powershell
cargo run --release --example phone_specs apple_iphone_15_pro_max-12548
```

### 4. Fetch Multiple Phones
```powershell
cargo run --release --example fetch_phones
```

### 5. Multi-Brand Comparison
```powershell
cargo run --release --example multi_brand_comparison
```

## Finding Phone IDs

Phone IDs come from GSMArena URLs:
- URL: `https://www.gsmarena.com/apple_iphone_15_pro_max-12548.php`
- ID: `apple_iphone_15_pro_max-12548`

## Popular Phone IDs

```
apple_iphone_15_pro_max-12548
apple_iphone_15_pro-12559
apple_iphone_15-12559
samsung_galaxy_s24_ultra-12771
samsung_galaxy_s24-12771
google_pixel_8_pro-12546
google_pixel_8-12546
oneplus_12-12712
xiaomi_14_ultra-12764
```

## Using as Library

```rust
use gsmarena;

fn main() {
    let phone_id = "apple_iphone_15-12559";
    let specs = gsmarena::get_specification(phone_id);
    
    // Get as JSON string
    let json = gsmarena::get_specification_json(phone_id);
    println!("{}", json);
}
```

## Output

All examples save results to JSON files in the current directory.
