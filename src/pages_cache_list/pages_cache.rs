use my_azure_storage_sdk::page_blob::consts::BLOB_PAGE_SIZE;

use super::{CachedPage, CachedPagesList};

pub struct PagesCache {
    cached_pages: Vec<CachedPagesList>,
}

impl PagesCache {
    pub fn new() -> Self {
        Self {
            cached_pages: Vec::new(),
        }
    }

    pub fn add_interval_to_cache(
        &mut self,
        from_page_id: usize,
        to_page_id: usize,
        max_pages_amount: usize,
    ) {
        self.cached_pages.push(CachedPagesList::new(
            max_pages_amount,
            from_page_id,
            to_page_id,
        ));
    }

    pub fn update_cache(&mut self, start_page: usize, payload: &[u8]) {
        if payload.is_empty() {
            return;
        }

        // We expect payload to be page-aligned; partial pages are not cached.
        if payload.len() % BLOB_PAGE_SIZE != 0 {
            return;
        }

        let payload = payload.to_vec();

        for cache in &mut self.cached_pages {
            cache.insert(start_page, payload.clone());
        }
    }

    pub fn get(&self, page_no: usize) -> Option<&CachedPage> {
        for cache in &self.cached_pages {
            if let Some(page) = cache.get_by_page_no(page_no) {
                return Some(page);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::PagesCache;
    use my_azure_storage_sdk::page_blob::consts::BLOB_PAGE_SIZE;

    #[test]
    fn test_cache_insert_and_get_within_interval() {
        let mut cache = PagesCache::new();
        cache.add_interval_to_cache(0, 2, 10);

        let mut payload = vec![1u8; BLOB_PAGE_SIZE * 2];
        cache.update_cache(0, payload.as_slice());

        let page0 = cache.get(0).unwrap();
        assert_eq!(page0.get_payload(), [1u8; BLOB_PAGE_SIZE].as_slice());

        let page1 = cache.get(1).unwrap();
        assert_eq!(page1.get_payload(), [1u8; BLOB_PAGE_SIZE].as_slice());
    }

    #[test]
    fn test_cache_skips_out_of_interval_pages() {
        let mut cache = PagesCache::new();
        cache.add_interval_to_cache(5, 6, 10);

        let payload = vec![2u8; BLOB_PAGE_SIZE * 2];
        cache.update_cache(0, payload.as_slice());

        assert!(cache.get(0).is_none());
        assert!(cache.get(5).is_none());
    }

    #[test]
    fn test_non_aligned_payload_is_ignored() {
        let mut cache = PagesCache::new();
        cache.add_interval_to_cache(0, 1, 10);

        let payload = vec![3u8; BLOB_PAGE_SIZE + 1];
        cache.update_cache(0, payload.as_slice());

        assert!(cache.get(0).is_none());
    }

    #[test]
    fn test_empty_payload_is_ignored() {
        let mut cache = PagesCache::new();
        cache.add_interval_to_cache(0, 1, 10);

        let payload = vec![];
        cache.update_cache(0, payload.as_slice());

        assert!(cache.get(0).is_none());
    }

    #[test]
    fn test_multiple_intervals_are_updated_independently() {
        let mut cache = PagesCache::new();
        cache.add_interval_to_cache(0, 1, 10);
        cache.add_interval_to_cache(5, 6, 10);

        let payload_a = vec![4u8; BLOB_PAGE_SIZE * 2];
        let payload_b = vec![5u8; BLOB_PAGE_SIZE * 2];

        cache.update_cache(0, payload_a.as_slice());
        cache.update_cache(5, payload_b.as_slice());

        assert_eq!(
            cache.get(0).unwrap().get_payload(),
            [4u8; BLOB_PAGE_SIZE].as_slice()
        );
        assert_eq!(
            cache.get(5).unwrap().get_payload(),
            [5u8; BLOB_PAGE_SIZE].as_slice()
        );
    }
}
