use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::{PageFromCacheResult, PagesInterval};

pub struct CachedPage {
    payload: Vec<u8>,
    created: DateTimeAsMicroseconds,
    pub page_no: usize,
}

pub struct PagesList {
    by_page_no: BTreeMap<usize, Arc<CachedPage>>,
    by_date_time: BTreeMap<i64, Arc<CachedPage>>,
    max_pages_amount: usize,
    page_size: usize,
}

impl PagesList {
    pub fn new(max_pages_amount: usize, page_size: usize) -> Self {
        Self {
            by_page_no: BTreeMap::new(),
            by_date_time: BTreeMap::new(),
            max_pages_amount,
            page_size,
        }
    }

    pub fn clear(&mut self) {
        self.by_page_no.clear();
        self.by_date_time.clear();
    }

    fn gc(&mut self) {
        while self.by_page_no.len() > self.max_pages_amount {
            self.remove_earliest();
        }
    }

    pub fn insert(&mut self, page_no: usize, payload: Vec<u8>) {
        if payload.len() != self.page_size {
            panic!(
                "Payload must be exact {} size. Payload with size {} is trying to be inserted",
                self.page_size,
                payload.len()
            )
        }
        let page = CachedPage {
            created: DateTimeAsMicroseconds::now(),
            page_no,
            payload,
        };

        let page = Arc::new(page);

        self.by_page_no.insert(page.page_no, page.clone());
        self.by_date_time
            .insert(page.created.unix_microseconds, page.clone());

        self.gc();
    }

    pub fn get_from_cache(
        &self,
        start_page_no: usize,
        pages_amount: usize,
    ) -> Vec<PageFromCacheResult> {
        let mut result = Vec::new();

        let mut missing: Option<PagesInterval> = None;

        let mut cached: Option<PagesInterval> = None;

        for page_no in start_page_no..start_page_no + pages_amount {
            if self.by_page_no.get(&page_no).is_some() {
                if missing.is_some() {
                    let mut missing_page_result = None;
                    std::mem::swap(&mut missing_page_result, &mut missing);

                    if let Some(missing_page_result) = missing_page_result {
                        result.push(PageFromCacheResult::MissingInterval(missing_page_result))
                    }
                }

                if let Some(cached_interval) = &mut cached {
                    cached_interval.pages_amount += 1
                } else {
                    cached = Some(PagesInterval::new(page_no))
                }
            } else {
                if cached.is_some() {
                    let mut cached_interval_result = None;
                    std::mem::swap(&mut cached_interval_result, &mut cached);

                    if let Some(cached_interval_result) = cached_interval_result {
                        result.push(PageFromCacheResult::CachedInterval(cached_interval_result))
                    }
                }

                if let Some(missing_interval) = &mut missing {
                    missing_interval.pages_amount += 1
                } else {
                    missing = Some(PagesInterval::new(page_no))
                }
            }
        }

        let mut missing_result = None;
        std::mem::swap(&mut missing_result, &mut missing);

        if let Some(interval) = missing_result {
            result.push(PageFromCacheResult::MissingInterval(interval))
        }

        let mut cached_interval = None;
        std::mem::swap(&mut cached_interval, &mut cached);

        if let Some(cached_interval) = cached_interval {
            result.push(PageFromCacheResult::CachedInterval(cached_interval))
        }

        return result;
    }

    fn get_earliest_microseconds(&self) -> Option<i64> {
        for itm in self.by_date_time.keys() {
            return Some(*itm);
        }

        None
    }

    pub fn remove_earliest(&mut self) -> Option<Arc<CachedPage>> {
        let microseconds = self.get_earliest_microseconds()?;

        let result = self.by_date_time.remove(&microseconds);

        if let Some(itm) = &result {
            self.by_page_no.remove(&itm.page_no);
        }

        result
    }

    pub fn get_by_page_no(&self, page_no: usize) -> Option<&[u8]> {
        let result = self.by_page_no.get(&page_no)?;
        result.payload.as_slice().into()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_we_have_whole_payload() {
        let mut pages_list = PagesList::new(10, 4);

        pages_list.insert(1, vec![1u8, 1u8, 1u8, 1u8]);

        pages_list.insert(2, vec![2u8, 2u8, 2u8, 2u8]);
        pages_list.insert(3, vec![3u8, 3u8, 3u8, 3u8]);

        let pages_result = pages_list.get_from_cache(1, 3);

        assert_eq!(1, pages_result.len());

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(1, interval.start_page_no);
            assert_eq!(3, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_we_have_missing_end() {
        let mut pages_list = PagesList::new(10, 4);

        pages_list.insert(1, vec![1u8, 1u8, 1u8, 1u8]);
        pages_list.insert(2, vec![2u8, 2u8, 2u8, 2u8]);
        pages_list.insert(3, vec![3u8, 3u8, 3u8, 3u8]);

        let pages_result = pages_list.get_from_cache(2, 3);

        assert_eq!(2, pages_result.len());

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(2, interval.start_page_no);
            assert_eq!(2, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(1).unwrap() {
            assert_eq!(4, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_we_have_missing_beginning() {
        let mut pages_list = PagesList::new(10, 4);

        pages_list.insert(1, vec![1u8, 1u8, 1u8, 1u8]);
        pages_list.insert(2, vec![2u8, 2u8, 2u8, 2u8]);
        pages_list.insert(3, vec![3u8, 3u8, 3u8, 3u8]);

        let pages_result = pages_list.get_from_cache(0, 4);

        assert_eq!(2, pages_result.len());

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(0, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(1).unwrap() {
            assert_eq!(1, interval.start_page_no);
            assert_eq!(3, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_we_have_all_elements_missing_before() {
        let mut pages_list = PagesList::new(10, 4);

        pages_list.insert(5, vec![5u8, 5u8, 5u8, 5u8]);
        pages_list.insert(6, vec![6u8, 6u8, 6u8, 6u8]);
        pages_list.insert(7, vec![7u8, 7u8, 7u8, 7u8]);

        let pages_result = pages_list.get_from_cache(1, 3);

        assert_eq!(1, pages_result.len());

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(1, interval.start_page_no);
            assert_eq!(3, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_we_have_all_elements_missing_after() {
        const PAGE_SIZE: usize = 4;
        let mut pages_list = PagesList::new(10, PAGE_SIZE);

        pages_list.insert(5, vec![5u8; PAGE_SIZE]);
        pages_list.insert(6, vec![6u8; PAGE_SIZE]);
        pages_list.insert(7, vec![7u8; PAGE_SIZE]);

        let pages_result = pages_list.get_from_cache(8, 3);

        assert_eq!(1, pages_result.len());

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(8, interval.start_page_no);
            assert_eq!(3, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_we_have_missing_elements_in_the_middle() {
        const PAGE_SIZE: usize = 4;
        let mut pages_list = PagesList::new(10, PAGE_SIZE);

        pages_list.insert(5, vec![5u8; PAGE_SIZE]);
        pages_list.insert(6, vec![6u8; PAGE_SIZE]);
        pages_list.insert(7, vec![7u8; PAGE_SIZE]);

        pages_list.insert(9, vec![9u8; PAGE_SIZE]);
        pages_list.insert(10, vec![10u8; PAGE_SIZE]);

        let pages_result = pages_list.get_from_cache(5, 6);

        assert_eq!(3, pages_result.len());

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(5, interval.start_page_no);
            assert_eq!(3, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(1).unwrap() {
            assert_eq!(8, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(2).unwrap() {
            assert_eq!(9, interval.start_page_no);
            assert_eq!(2, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_we_have_missing_elements_at_beginning_and_in_the_middle() {
        const PAGE_SIZE: usize = 4;
        let mut pages_list = PagesList::new(10, PAGE_SIZE);

        pages_list.insert(5, vec![5u8; PAGE_SIZE]);
        pages_list.insert(6, vec![6u8; PAGE_SIZE]);
        pages_list.insert(7, vec![7u8; PAGE_SIZE]);

        pages_list.insert(9, vec![9u8; PAGE_SIZE]);
        pages_list.insert(10, vec![10u8; PAGE_SIZE]);

        let pages_result = pages_list.get_from_cache(4, 7);

        assert_eq!(4, pages_result.len());

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(4, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(1).unwrap() {
            assert_eq!(5, interval.start_page_no);
            assert_eq!(3, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(2).unwrap() {
            assert_eq!(8, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(3).unwrap() {
            assert_eq!(9, interval.start_page_no);
            assert_eq!(2, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_we_have_missing_elements_at_beginning_in_the_middle_in_the_end() {
        const PAGE_SIZE: usize = 4;
        let mut pages_list = PagesList::new(10, PAGE_SIZE);

        pages_list.insert(5, vec![5u8; PAGE_SIZE]);
        pages_list.insert(6, vec![6u8; PAGE_SIZE]);
        pages_list.insert(7, vec![7u8; PAGE_SIZE]);

        pages_list.insert(9, vec![9u8; PAGE_SIZE]);
        pages_list.insert(10, vec![10u8; PAGE_SIZE]);

        let pages_result = pages_list.get_from_cache(5, 7);

        assert_eq!(4, pages_result.len());

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(5, interval.start_page_no);
            assert_eq!(3, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(1).unwrap() {
            assert_eq!(8, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(2).unwrap() {
            assert_eq!(9, interval.start_page_no);
            assert_eq!(2, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(3).unwrap() {
            assert_eq!(11, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }

    #[test]
    fn test_we_have_missing_elements_in_the_middle_in_the_end() {
        const PAGE_SIZE: usize = 4;
        let mut pages_list = PagesList::new(10, PAGE_SIZE);

        pages_list.insert(5, vec![5u8; PAGE_SIZE]);
        pages_list.insert(6, vec![6u8; PAGE_SIZE]);
        pages_list.insert(7, vec![7u8; PAGE_SIZE]);

        pages_list.insert(9, vec![9u8; PAGE_SIZE]);
        pages_list.insert(10, vec![10u8; PAGE_SIZE]);

        let pages_result = pages_list.get_from_cache(4, 8);

        assert_eq!(5, pages_result.len());

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(0).unwrap() {
            assert_eq!(4, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(1).unwrap() {
            assert_eq!(5, interval.start_page_no);
            assert_eq!(3, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(2).unwrap() {
            assert_eq!(8, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::CachedInterval(interval) = pages_result.get(3).unwrap() {
            assert_eq!(9, interval.start_page_no);
            assert_eq!(2, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }

        if let PageFromCacheResult::MissingInterval(interval) = pages_result.get(4).unwrap() {
            assert_eq!(11, interval.start_page_no);
            assert_eq!(1, interval.pages_amount);
        } else {
            panic!("Should not be here");
        }
    }
}
