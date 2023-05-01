use my_azure_storage_sdk::{
    blob::BlobProperties,
    page_blob::{consts::BLOB_PAGE_SIZE, PageBlobProperties},
};

use crate::{pages_list::PagesCache, PagesCacheIntervals};

pub struct PageBlobCachedData {
    pub page_blob_properties: Option<PageBlobProperties>,
    pub cached_pages: Option<PagesCache>,
    pub pages_to_write: PagesCacheIntervals,
}

impl PageBlobCachedData {
    pub fn new(max_cached_pages_amount: usize, page_size: usize) -> Self {
        let cached_pages = if max_cached_pages_amount == 0 {
            None
        } else {
            Some(PagesCache::new(max_cached_pages_amount, page_size))
        };

        Self {
            page_blob_properties: None,
            cached_pages,
            pages_to_write: PagesCacheIntervals::new(),
        }
    }
    pub fn update_pages_amount(&mut self, pages_amount: usize) {
        let blob_size = pages_amount * BLOB_PAGE_SIZE;
        if let Some(page_blob_properties) = &mut self.page_blob_properties {
            page_blob_properties.blob_properties.blob_size = blob_size;
        } else {
            self.page_blob_properties = Some(PageBlobProperties::new(BlobProperties { blob_size }));
        }
    }

    pub fn update_blob_properties(&mut self, blob_properties: PageBlobProperties) {
        self.page_blob_properties = Some(blob_properties);
    }
}
