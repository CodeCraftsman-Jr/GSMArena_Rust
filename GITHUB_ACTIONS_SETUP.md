# GitHub Actions Setup for ScrapingBee Hybrid Scraper

## Quick Setup

### 1. Add GitHub Secrets

Go to your repository: **Settings → Secrets and variables → Actions → New repository secret**

Add the following secrets:

```
MONGO_DB_USERNAME=your_mongodb_username
MONGO_DB_PASSWORD=your_mongodb_password
MONGO_DB_DATABASE_NAME=your_database_name
MONGO_DB_DOMAIN_NAME=your_cluster_domain

SCRAPINGBEE_API_KEYS=key1,key2,key3
```

### 2. ScrapingBee API Keys Format

**Option A: Use GitHub Secret (Recommended for security)**
- Add `SCRAPINGBEE_API_KEYS` as a repository secret
- Format: Comma-separated keys without spaces
- Example: `ABC123KEY1,DEF456KEY2,GHI789KEY3`

**Option B: Enter keys manually when running**
- Leave the secret empty
- When triggering the workflow, paste keys in the input field
- Same format: comma-separated

### 3. Run the Workflow

1. Go to **Actions** tab in your repository
2. Select **"Scrape GSMArena Phones (ScrapingBee Hybrid)"**
3. Click **"Run workflow"**
4. Configure parameters:
   - **Max brands**: Leave empty for all, or enter a number (e.g., `5`)
   - **Phones per brand**: Leave empty for all, or enter a number (e.g., `50`)
   - **Collection name**: Default is `gsmarena_phones`
   - **Hybrid batch size**: Default is `10` (10 rate-limited + 10 ScrapingBee)
   - **ScrapingBee API keys**: Leave empty to use secret, or paste comma-separated keys
5. Click **"Run workflow"**

## Workflow Features

✅ **Automatic caching** - Faster builds with Cargo cache  
✅ **Flexible configuration** - Customize via UI inputs  
✅ **Dual key input** - Use secrets or manual entry  
✅ **Hybrid mode** - Alternates between free and paid requests  
✅ **Smart skipping** - Pre-loads existing phones to avoid duplicates  
✅ **Logs & artifacts** - Download logs after completion  
✅ **Summary report** - View status in Actions summary  

## Configuration Options

| Parameter | Description | Default |
|-----------|-------------|---------|
| `max_brands` | Number of brands to scrape | ALL |
| `phones_per_brand` | Phones per brand | ALL |
| `collection_name` | MongoDB collection | `gsmarena_phones` |
| `hybrid_batch_size` | Phones per batch cycle | `10` |
| `scrapingbee_api_keys` | API keys (comma-separated) | From secret |

## Cost Estimation

For full scrape (~250 brands, ~6,250 phones):
- **Brand fetching**: 250 × 1 credit = 250 credits
- **Phone lists**: 250 × ~3 credits = 750 credits  
- **Phone specs** (50% via ScrapingBee): 3,125 × 1 credit = 3,125 credits
- **Total**: ~4,125 credits per full scrape

With 3 API keys × 1,000 credits = 3,000 free credits/month, you can do partial scrapes or use the rate-limited fallback for the rest.

## Monitoring

- View real-time logs in the Actions tab
- Download log artifacts after completion (retained for 7 days)
- Check the summary for quick status overview

## Troubleshooting

**Error: "SCRAPINGBEE_API_KEYS not set"**
- Ensure the secret is properly configured in GitHub Settings
- Or provide keys manually in the workflow input field

**Build fails**
- Check that all MongoDB secrets are configured
- Verify secret names match exactly (case-sensitive)

**Scraping stops early**
- May have exhausted API credits - check ScrapingBee dashboard
- Will automatically fall back to rate-limited mode

## Security Notes

🔒 **Never commit API keys to the repository**  
🔒 **Use GitHub Secrets for permanent keys**  
🔒 **Manual input is for temporary/testing purposes**  
🔒 **Keys in workflow inputs are visible in logs**  
