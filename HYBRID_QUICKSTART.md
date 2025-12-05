# Quick Start - Hybrid ScrapingBee Scraper

## What You Built

A smart scraper that **alternates between free and paid requests**:
- **10 phones**: Free rate-limited requests (500ms delay)
- **10 phones**: Paid ScrapingBee requests (fast, no delay)
- **Repeat**: Alternates to save credits while maintaining speed

## Quick Test Run

### 1. Add API Keys to .env

```bash
SCRAPINGBEE_API_KEYS=your_api_key_1,your_api_key_2
```

### 2. Test with 2 Brands

```bash
# Fetch 2 brands, 20 phones each = 40 phones total
# Uses: 20 rate-limited + 20 ScrapingBee = ~21 credits
cargo run --release --bin scrape_phonelists_scrapingbee 2 20
```

### 3. Output Example

```
GSMArena Hybrid Scraper - ScrapingBee + Rate Limited
====================================================

Configuration:
  Collection: gsmarena_phones
  Max brands: 2
  Max phones per brand: 20
  Skip existing: true
  Hybrid batch size: 10 phones
  Rate limit delay: 500ms
  Mode: 10 rate-limited + 10 ScrapingBee (alternating)

Initializing ScrapingBee...
✓ Loaded 2 ScrapingBee API key(s)
✓ Using 2 API key(s) with rotation

Connecting to MongoDB...
✓ Connected to MongoDB

Fetching brands...
Fetching brands through ScrapingBee... ✓
✓ Found 125 brands

[1/2] Processing: Apple (113 devices)
----------------------------------------------------------------------
  Fetching phone list (ScrapingBee)... ✓ Found 113 phones
  Fetching specifications (hybrid mode):
    [1/20] Apple iPhone 15 Pro Max [RL] ✓
    [2/20] Apple iPhone 15 Pro [RL] ✓
    [3/20] Apple iPhone 15 Plus [RL] ✓
    ...
    [10/20] Apple iPhone 14 [RL] ✓
    [11/20] Apple iPhone 13 Pro [SB] ✓
    [12/20] Apple iPhone 13 [SB] ✓
    ...
    [20/20] Apple iPhone 12 mini [SB] ✓
  ✓ Saved 20 phones with full specifications

[2/2] Processing: Samsung (553 devices)
----------------------------------------------------------------------
  Fetching phone list (ScrapingBee)... ✓ Found 553 phones
  Fetching specifications (hybrid mode):
    [1/20] Samsung Galaxy S24 Ultra [RL] ✓
    [2/20] Samsung Galaxy S24+ [RL] ✓
    ...
    [20/20] Samsung Galaxy S23 FE [SB] ✓
  ✓ Saved 20 phones with full specifications

======================================================================
✓ Scraping Complete!
======================================================================
Statistics:
  Brands processed: 2/2
  Brands failed: 0
  Total phones found: 666
  Phones with specs saved: 40
  Phones skipped (existing): 0
  Failed: 0

Database:
  Collection: gsmarena_phones
  Previous count: 0
  Current count: 40
  Net change: +40

Hybrid Method:
  Alternating: 10 rate-limited + 10 ScrapingBee per batch
  This saves API credits while maintaining speed
======================================================================
```

## Understanding the Output

- **[RL]** = Rate-Limited (free, 500ms delay)
- **[SB]** = ScrapingBee (paid, fast)
- Pattern: 10 RL → 10 SB → 10 RL → 10 SB...

## Credit Calculation

For 40 phones with 10+10 batching:
- Phone lists: 2 credits (2 brands)
- Specs: 20 ScrapingBee + 20 rate-limited = **20 credits**
- **Total: ~22 credits for 40 complete phones**

Compare to all-ScrapingBee: Would cost ~42 credits

## Run Full Scrape

```bash
# Get all brands, all phones (will take hours and use ~500 credits)
cargo run --release --bin scrape_phonelists_scrapingbee

# Recommended: Start with 10 brands
cargo run --release --bin scrape_phonelists_scrapingbee 10
```

## If ScrapingBee Exhausted

When all API keys run out of credits, the scraper automatically switches to **rate-limited only** mode and continues:

```
⚠ ScrapingBee exhausted, switching to rate-limited only
[21/40] Samsung Galaxy Z Fold5 [RL] ✓
[22/40] Samsung Galaxy Z Flip5 [RL] ✓
```

No data is lost - it just runs slower!

## Next Steps

1. **Monitor credits**: Check ScrapingBee dashboard
2. **Adjust batch size**: `HYBRID_BATCH_SIZE=20` for larger batches
3. **Scale up**: Add more API keys for higher throughput
4. **Query data**: Check MongoDB for complete phone specifications

## Advantages

| Method | Speed | Cost | Coverage |
|--------|-------|------|----------|
| **Hybrid (this)** | Medium-Fast | ~500 credits | ~1,000 phones |
| Pure ScrapingBee | Very Fast | ~1,000 credits | ~150 phones |
| Pure Rate-Limited | Slow | Free | Unlimited* |

*Risk of blocking after ~1,000 phones
