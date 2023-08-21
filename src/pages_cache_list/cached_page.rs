use my_azure_storage_sdk::page_blob::consts::BLOB_PAGE_SIZE;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct CachedPage {
    payload: Vec<u8>,
    pub created: DateTimeAsMicroseconds,
    pub page_id: usize,
}

impl CachedPage {
    pub fn get_payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn get_pages_amount(&self) -> usize {
        self.payload.len() / BLOB_PAGE_SIZE
    }

    pub fn get_last_page_id(&self) -> usize {
        self.page_id + self.get_pages_amount()
    }
}
