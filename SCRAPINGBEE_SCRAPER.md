# ScrapingBee Hybrid Scraper

## Overview

This is a **hybrid scraper** that uses both **ScrapingBee API** and **rate-limited direct requests** to fetch **complete phone data with detailed specifications**. It alternates between methods to optimize API credit usage while maintaining speed.

## Key Features

- ✅ **Hybrid Approach**: Alternates between rate-limited (free) and ScrapingBee (paid) requests
- ✅ **Multiple API Key Rotation**: Automatically switches between API keys when one is exhausted  
- ✅ **Full Specifications**: Fetches complete phone specs with organized data structure
- ✅ **Configurable Batching**: Set how many phones use each method (default: 10 + 10)
- ✅ **Smart Fallback**: Switches to rate-limited only when ScrapingBee exhausted
- ✅ **Separate Binary**: Doesn't interfere with existing scrapers
- ✅ **MongoDB Storage**: Saves to `gsmarena_phones` collection with full specs

## How It Works

The scraper alternates in batches:
1. **First 10 phones**: Rate-limited direct requests (500ms delay, free)
2. **Next 10 phones**: ScrapingBee API requests (fast, uses credits)
3. **Next 10 phones**: Back to rate-limited
4. **Pattern repeats** until all phones processed

Example flow for 30 phones:
```
Phones 1-10:   [RL] [RL] [RL] [RL] [RL] [RL] [RL] [RL] [RL] [RL]
Phones 11-20:  [SB] [SB] [SB] [SB] [SB] [SB] [SB] [SB] [SB] [SB]
Phones 21-30:  [RL] [RL] [RL] [RL] [RL] [RL] [RL] [RL] [RL] [RL]
```

## Credit Usage

With 1,000 ScrapingBee credits and default batch size (10):
- **Brands list**: 1 credit → 125 brands
- **Phone lists**: ~375 credits → ~6,250 phone IDs (via ScrapingBee)
- **Specifications**: ~312 credits → ~312 full phone specs (50% via ScrapingBee)
- **Total**: ~688 credits for **~312 complete phones** + directory of 6,250 phones
- **Remaining**: 312 credits for future runs

**Without hybrid mode** (all ScrapingBee): Only ~150 phones with 1,000 credits
**With hybrid mode**: ~312 phones (2x more data!)

## Setup

### 1. Configure Environment Variables

Add to your `.env` file:

```bash
# ScrapingBee API Keys (comma-separated for rotation)
SCRAPINGBEE_API_KEYS=key1,key2,key3

# MongoDB Configuration
MONGO_DB_USERNAME=your_username
MONGO_DB_PASSWORD=your_password
MONGO_DB_DATABASE_NAME=baeonDBStage
MONGO_DB_DOMAIN_NAME=baeonncluster.oakjn89

# Collection name
COLLECTION_NAME=gsmarena_phones

# Hybrid mode configuration
HYBRID_BATCH_SIZE=10                # Phones per batch (default: 10)
DELAY_BETWEEN_PHONES_MS=500         # Delay for rate-limited requests
SKIP_EXISTING=true                  # Skip phones already in database

# Optional: Limit scope
MAX_BRANDS=
PHONES_PER_BRAND=
```

### 2. Build the Scraper

```bash
cargo build --release --bin scrape_phonelists_scrapingbee
```

## Usage

### Scrape All Brands with Full Specs (Hybrid Mode)

```bash
cargo run --release --bin scrape_phonelists_scrapingbee
```

### Scrape Limited Brands

```bash
# Scrape 5 brands, 20 phones each = 100 phones total
cargo run --release --bin scrape_phonelists_scrapingbee 5 20

# Or set environment variables
MAX_BRANDS=5 PHONES_PER_BRAND=20 cargo run --release --bin scrape_phonelists_scrapingbee
```

### Custom Batch Size

```bash
# Use 20+20 batches instead of 10+10
HYBRID_BATCH_SIZE=20 cargo run --release --bin scrape_phonelists_scrapingbee
```

## How It Works

1. **Loads API Keys**: Reads comma-separated keys from `SCRAPINGBEE_API_KEYS`
2. **Rotates Automatically**: When one key hits 429/403, switches to next key
3. **Fetches Brands**: Gets all 125 brands (1 credit)
4. **Fetches Phone Lists**: For each brand, fetches all paginated phone lists (~3 credits per brand)
5. **Saves to MongoDB**: Stores phone metadata in `gsmarena_phone_lists` collection
6. **Stops on Exhaustion**: When all API keys exhausted, reports progress and exits cleanly

## Output Structure

### MongoDB Document

```json
{
  "phone_id": "apple_iphone_15-12559",
  "name": "Apple iPhone 15",
  "brand": "Apple",
  "url": "https://www.gsmarena.com/apple_iphone_15-12559.php",
  "image_url": "https://www.gsmarena.com/images/...",
  "source": "gsmarena",
  "scraped_at": "2025-12-05T10:30:00Z"
}
```

### Terminal Output

```
GSMArena Phone Lists Scraper - ScrapingBee Only (Option B)
==========================================================

Configuration:
  Collection: gsmarena_phone_lists
  Max brands: ALL
  Mode: ScrapingBee only (no fallback)

Initializing ScrapingBee...
✓ Loaded 3 ScrapingBee API key(s)
✓ Using 3 API key(s) with rotation

Connecting to MongoDB...
✓ Connected to MongoDB

Current phone lists in database: 0

Fetching brands...
Fetching brands through ScrapingBee... ✓
✓ Found 125 brands

[1/125] Processing: Apple (113 devices)
----------------------------------------------------------------------
  Fetching phone list (ScrapingBee)... ✓ Found 113 phones
  Saving to MongoDB... ✓ Saved 113 phones

[2/125] Processing: Samsung (553 devices)
----------------------------------------------------------------------
  Fetching phone list (ScrapingBee)... ✓ Found 553 phones
  Saving to MongoDB... ✓ Saved 553 phones

...

⚠ All ScrapingBee API keys exhausted!
Processed 95/125 brands before exhaustion

======================================================================
✓ Scraping Complete!
======================================================================
Statistics:
  Brands processed: 95/125
  Brands failed: 0
  Total phones found: 5,234
  Phone lists saved: 5,234
  Failed saves: 0

Database:
  Collection: gsmarena_phone_lists
  Previous count: 0
  Current count: 5,234
  Net change: +5,234

Note: Only phone lists saved (no detailed specs)
Use this data to fetch specs separately with rate-limited scraper
======================================================================
```

## Next Steps

After collecting phone lists with ScrapingBee:

1. **Query MongoDB** to get phone IDs:
```javascript
db.gsmarena_phone_lists.find({}, {phone_id: 1, name: 1, brand: 1})
```

2. **Use Rate-Limited Scraper** to fetch detailed specs:
```bash
# Fetch specs for phones that don't have them yet
cargo run --release --bin scrape_to_mongodb_ratelimited
```

3. **Combine Data**: Use the phone lists as a master index, fetch specs gradually over multiple runs

## Error Handling

### API Key Exhaustion
```
⚠ API key 1 exhausted (status 429), switching...
⚠ API key 2 exhausted (status 403), switching...
⚠ All 3 API keys exhausted
```
**Solution**: Wait for credits to reset or add more API keys

### Network Errors
```
⚠ Request failed, trying next key...
```
**Solution**: Automatic retry with next API key

### MongoDB Errors
```
✗ Error saving Apple iPhone 15: connection timeout
```
**Solution**: Check MongoDB connection and credentials

## Advantages Over Rate-Limited Scraper

| Feature | ScrapingBee (This) | Rate-Limited |
|---------|-------------------|--------------|
| Speed | Fast (no delays) | Slow (500ms delays) |
| Blocking Risk | None (rotating IPs) | High after ~1000 phones |
| Coverage | 6,250+ phones with 1000 credits | Unlimited but risky |
| Cost | Paid credits | Free but time-consuming |
| Best For | Getting complete directory | Fetching detailed specs |

## Troubleshooting

### No API Keys Loaded
```
Error: SCRAPINGBEE_API_KEYS not set
```
Add keys to `.env`: `SCRAPINGBEE_API_KEYS=key1,key2,key3`

### MongoDB Connection Failed
```
Error: MONGO_DB_USERNAME not set
```
Ensure all MongoDB environment variables are set in `.env`

### Build Errors
```bash
cargo clean
cargo build --release --bin scrape_phonelists_scrapingbee
```

## Credits Estimation

- **1 brand list**: 1 credit
- **125 brands**: 1 credit total
- **1 phone list page**: 1 credit (up to 50 phones)
- **Average 3 pages per brand**: 3 credits × 125 = 375 credits
- **Total for all phone lists**: ~376 credits
- **Remaining for future runs**: 624 credits with 1000-credit plan

## Integration with Existing Scrapers

This scraper is **completely separate**:
- Different binary: `scrape_phonelists_scrapingbee` vs `scrape_to_mongodb_ratelimited`
- Different collection: `gsmarena_phone_lists` vs `gsmarena_phones`
- Different purpose: Directory vs Full specs
- No code conflicts: Both can coexist

Use them together:
1. Run **this scraper** first to get all phone IDs quickly
2. Run **rate-limited scraper** to fetch specs for specific phones
3. Query MongoDB to see which phones still need specs
