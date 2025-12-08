# Phone List Collection - Request Optimization

## Overview

The hybrid scraper now uses **two MongoDB collections** for intelligent request optimization:

1. **`gsmarena_phones`** - Stores complete phone specifications (13 categories + raw JSON)
2. **`gsmarena_phone_list`** - Tracks completion status of each phone

## How It Saves Requests

### Traditional Approach (Without Phone List)
- Every run fetches specs for ALL phones
- Wastes API credits re-fetching existing data
- No way to track partial completions

### Optimized Approach (With Phone List)
- **Pre-loads completion status** at startup → O(1) lookups
- **Skips complete phones** → Saves 100% of requests for existing data
- **Tracks incomplete phones** → Can resume interrupted scrapes
- **Marks phones complete** → Automatic after successful spec save

## Phone List Collection Schema

```javascript
{
  phone_id: "samsung_galaxy_s24_ultra-12345",
  name: "Samsung Galaxy S24 Ultra",
  brand: "Samsung",
  url: "https://www.gsmarena.com/samsung_galaxy_s24_ultra-12345.php",
  image_url: "https://fdn2.gsmarena.com/vv/bigpic/samsung-galaxy-s24-ultra.jpg",
  is_complete: false,  // ← Key field!
  created_at: "2025-12-05T10:30:00Z",
  updated_at: "2025-12-05T10:30:00Z"
}
```

## Workflow

### Startup
1. Connect to MongoDB
2. Load all phone IDs where `is_complete: true` into HashSet
3. Display count: "Found 3,252 complete phones to skip"

### Per Phone
1. Check if `phone_id` exists in HashSet → **Skip if complete**
2. If not complete:
   - Create/update entry in `gsmarena_phone_list` with `is_complete: false`
   - Fetch specifications (hybrid mode: rate-limited or ScrapingBee)
   - Save specs to `gsmarena_phones`
   - Mark as `is_complete: true` in `gsmarena_phone_list`
   - Add to in-memory HashSet for current run

### Statistics
```
Current phones in specs database: 3252
Current phones in list database: 3500
Loading complete phone IDs from phone list... ✓ Found 3252 complete phones to skip

... scraping ...

Database:
  Specs collection: gsmarena_phones
    Previous count: 3252
    Current count: 3270
    Net change: +18
  Phone list collection: gsmarena_phone_list
    Total phones: 3518
    Complete: 3270
    Incomplete: 248  ← Need to fetch these
```

## Benefits

### Request Savings
- **First run**: Fetches specs for all phones (~6,250 phones)
- **Second run**: Skips 6,250 complete phones, fetches 0 new = **100% request savings**
- **With new phones**: Only fetches specs for new/incomplete phones

### Credit Efficiency
Example: 100 new phones released since last run
- Without optimization: 6,350 requests (6,250 old + 100 new)
- With optimization: 100 requests (only new phones)
- **Savings: 6,250 requests = ~6,250 ScrapingBee credits saved**

### Resumability
- Run interrupted at 2,000/6,250 phones
- Next run: Automatically resumes at phone 2,001
- No wasted re-scraping

## Configuration

Add to `.env`:

```env
# Collections
COLLECTION_NAME=gsmarena_phones
PHONE_LIST_COLLECTION_NAME=gsmarena_phone_list

# Behavior
SKIP_EXISTING=true  # Set to false to re-scrape everything
```

## MongoDB Queries

### Check completion status
```javascript
// Total phones
db.gsmarena_phone_list.countDocuments()

// Complete phones
db.gsmarena_phone_list.countDocuments({ is_complete: true })

// Incomplete phones
db.gsmarena_phone_list.countDocuments({ is_complete: false })

// Phones for a specific brand
db.gsmarena_phone_list.find({ brand: "Samsung", is_complete: false })
```

### Reset completion (force re-scrape)
```javascript
// Reset all
db.gsmarena_phone_list.updateMany({}, { $set: { is_complete: false } })

// Reset specific brand
db.gsmarena_phone_list.updateMany(
  { brand: "Samsung" },
  { $set: { is_complete: false } }
)
```

## Performance

### Without Phone List (Old Method)
- 6,250 phones × 1 MongoDB query each = 6,250 DB queries per run
- Re-fetches specs for all phones every run
- Slow and wasteful

### With Phone List (New Method)
- **1 query** to load all complete phone IDs at startup
- **O(1) HashSet lookup** per phone (in-memory)
- **100× faster** existence checking
- **Saves thousands of API requests** on subsequent runs

## Example Output

```
[1/2] Processing: Samsung (462 devices)
----------------------------------------------------------------------
  Fetching phone list (ScrapingBee)... ✓ Found 462 phones
  Fetching specifications (hybrid mode):
    [1/5] Samsung Galaxy S24 Ultra - Already complete, skipping
    [2/5] Samsung Galaxy S24+ - Already complete, skipping
    [3/5] Samsung Galaxy S24 [RL] ✓
    [4/5] Samsung Galaxy Z Fold6 5G - Already complete, skipping
    [5/5] Samsung Galaxy Z Flip6 [SB] ✓
  ✓ Saved 2 phones with full specifications
```

## API Integration

This works seamlessly with GitHub Actions:

```yaml
env:
  COLLECTION_NAME: gsmarena_phones
  PHONE_LIST_COLLECTION_NAME: gsmarena_phone_list
  SKIP_EXISTING: 'true'
```

Run monthly → Only scrapes new phones → Minimal credit usage!
