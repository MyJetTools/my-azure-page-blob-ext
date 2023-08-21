use std::{collections::BTreeMap, sync::Arc};

use my_azure_storage_sdk::page_blob::consts::BLOB_PAGE_SIZE;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::{index_by_date::IndexByDate, CachedPage};

pub struct CachedPagesList {
    by_page_no: BTreeMap<usize, Arc<CachedPage>>,
    index_by_date: IndexByDate,

    max_pages_amount: usize,
    from_page_id: usize,
    to_page_id: usize,
}

impl CachedPagesList {
    pub fn new(max_pages_amount: usize, from_page_id: usize, to_page_id: usize) -> Self {
        Self {
            by_page_no: BTreeMap::new(),
            index_by_date: IndexByDate::new(),
            max_pages_amount,
            from_page_id,
            to_page_id,
        }
    }

    pub fn clear(&mut self) {
        self.by_page_no.clear();
        self.index_by_date.clear();
    }

    fn gc(&mut self) {
        while self.by_page_no.len() > self.max_pages_amount {
            self.index_by_date.remove_earliest();
        }
    }

    pub fn insert(&mut self, page_no: usize, payload: Vec<u8>) {
        let mut no = 0;
        for page_id in page_no..page_no + payload.len() / BLOB_PAGE_SIZE {
            if self.from_page_id <= page_no && page_no <= self.to_page_id {
                no += 1;
                continue;
            }

            let offset = no * BLOB_PAGE_SIZE;

            let new_page = CachedPage {
                created: DateTimeAsMicroseconds::now(),
                page_id,
                payload: (&payload[offset..offset + BLOB_PAGE_SIZE]).to_vec(),
            };

            let new_page = Arc::new(new_page);

            let old_page = self.by_page_no.insert(new_page.page_id, new_page.clone());

            if let Some(old_page) = old_page {
                self.index_by_date.remove(&old_page);
            }

            self.index_by_date.add(new_page);

            no += 1;
        }

        self.gc();
    }

    pub fn get_by_page_no(&self, page_no: usize) -> Option<&[u8]> {
        let result = self.by_page_no.get(&page_no)?;
        result.get_payload().into()
    }
}
