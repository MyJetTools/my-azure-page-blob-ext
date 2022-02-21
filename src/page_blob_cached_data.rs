use crate::pages_cache::PagesCache;

pub struct PageBlobCachedData {
    pub pages_amount: Option<usize>,
    pub cached_pages: Option<PagesCache>,
}

impl PageBlobCachedData {
    pub fn new(max_cached_pages_amount: usize, page_size: usize) -> Self {
        let cached_pages = if max_cached_pages_amount == 0 {
            None
        } else {
            Some(PagesCache::new(max_cached_pages_amount, page_size))
        };

        Self {
            pages_amount: None,
            cached_pages,
        }
    }

    pub fn get_pages_amount(&self) -> Option<usize> {
        self.pages_amount
    }

    pub fn set_pages_amount(&mut self, pages_amount: usize) {
        self.pages_amount = Some(pages_amount);
    }

    pub fn reset_cache(&mut self) {
        self.pages_amount = None;

        if let Some(cache) = &mut self.cached_pages {
            cache.clear();
        }
    }
}
