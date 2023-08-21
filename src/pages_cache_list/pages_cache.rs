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
        todo!("Implement");
    }

    pub fn get(&self, page_no: usize) -> Option<&CachedPage> {
        todo!("Implement");
    }
}
