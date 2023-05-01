use super::{PageFromCacheResult, PagesList};

pub struct PagesCache {
    max_pages_to_cache: usize,
    page_size: usize,
    pages: PagesList,
}

impl PagesCache {
    pub fn new(max_pages_to_cache: usize, page_size: usize) -> Self {
        Self {
            max_pages_to_cache,
            page_size,
            pages: PagesList::new(max_pages_to_cache, page_size),
        }
    }

    pub fn clear(&mut self) {
        self.pages.clear();
    }

    pub fn save_to_cache(&mut self, start_page: usize, payload: &[u8]) {
        let pages_amount = payload.len() / self.page_size;

        let offset = if pages_amount < self.max_pages_to_cache {
            0
        } else {
            pages_amount - self.max_pages_to_cache
        };

        let mut payload_offset = offset * self.page_size;

        for page_no in start_page + offset..start_page + pages_amount {
            let payload_to_cache = &payload[payload_offset..payload_offset + self.page_size];
            self.pages.insert(page_no, payload_to_cache.to_vec());
            payload_offset += self.page_size;
        }
    }

    pub fn get(&self, start_page_no: usize, pages_amount: usize) -> Vec<PageFromCacheResult> {
        return self.pages.get_from_cache(start_page_no, pages_amount);
    }

    pub fn get_payload(&self, page_no: usize) -> Option<&[u8]> {
        self.pages.get_by_page_no(page_no)
    }
}
