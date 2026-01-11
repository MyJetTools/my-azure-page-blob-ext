## my-azure-page-blob-ext

Rust helpers on top of `my-azure-storage-sdk` page blobs:
- Retry wrapper: `MyAzurePageBlobStorageWithRetries` adds configurable retry count and delay around every page-blob call.
- Optional in-memory cache (feature `blob_with_cache`): `MyAzurePageBlobWithCache` keeps recent pages and pending writes, reduces fetches, and caches blob properties.
- Utilities: helpers for page sizing and padding (`utils`).

### Features
- `blob_with_cache` (off by default) enables cache modules and re-exports cache types.

### Quick start
Add to your Cargo.toml:
```
my-azure-page-blob-ext = { path = ".", features = ["blob_with_cache"] }
```

Wrap an existing page blob client with retries:
```rust
use my_azure_page_blob_ext::MyAzurePageBlobStorageWithRetries;
use my_azure_storage_sdk::page_blob::AzurePageBlobStorage;
use std::time::Duration;

let page_blob = AzurePageBlobStorage::new(/* storage creds and names */);
let client = MyAzurePageBlobStorageWithRetries::new(page_blob, 3, Duration::from_secs(1));
```

Enable caching (requires `blob_with_cache` feature):
```rust
use my_azure_page_blob_ext::MyAzurePageBlobWithCache;
use my_azure_storage_sdk::page_blob::AzurePageBlobStorage;

let page_blob = AzurePageBlobStorage::new(/* storage creds and names */);
let cached = MyAzurePageBlobWithCache::new(page_blob);
// Use cached as a MyAzurePageBlobStorage implementation.
```

### Notes
- Caching keeps recently read/written pages in memory; eviction is interval-based with tests covering edge cases.
- Retry wrapper is transparent and keeps the same trait surface as `MyAzurePageBlobStorage`.
