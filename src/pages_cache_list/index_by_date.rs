use std::{collections::BTreeMap, sync::Arc};

use super::CachedPage;

pub struct IndexByDate {
    data: BTreeMap<i64, Vec<Arc<CachedPage>>>,
}

impl IndexByDate {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, page: Arc<CachedPage>) {
        let key = page.created.unix_microseconds;

        if !self.data.contains_key(&key) {
            self.data.insert(key, Vec::new());
        }
        let value = self.data.get_mut(&key).unwrap();
        value.push(page);
    }

    pub fn remove(&mut self, page: &CachedPage) {
        let key = page.created.unix_microseconds;
        let value = self.data.get_mut(&key).unwrap();
        value.retain(|x| x.page_id != page.page_id);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn get_earliest_microseconds(&self) -> Option<i64> {
        for itm in self.data.keys() {
            return Some(*itm);
        }

        None
    }

    pub fn remove_earliest(&mut self) -> Option<Vec<Arc<CachedPage>>> {
        let microseconds = self.get_earliest_microseconds()?;
        self.data.remove(&microseconds)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use crate::pages_cache_list::CachedPage;

    use super::IndexByDate;

    #[test]
    fn test_add_and_remove_index() {
        let mut index = IndexByDate::new();

        let page = CachedPage {
            created: DateTimeAsMicroseconds::new(5),
            page_id: 0,
            payload: vec![],
        };

        index.add(Arc::new(page));

        assert_eq!(index.data.len(), 1);

        let item = index.data.get(&5).unwrap();

        assert_eq!(item.len(), 1);
    }

    #[test]
    fn test_remove_earliest_returns_all_for_timestamp() {
        let mut index = IndexByDate::new();

        let page_a = Arc::new(CachedPage {
            created: DateTimeAsMicroseconds::new(5),
            page_id: 1,
            payload: vec![],
        });

        let page_b = Arc::new(CachedPage {
            created: DateTimeAsMicroseconds::new(5),
            page_id: 2,
            payload: vec![],
        });

        index.add(page_a.clone());
        index.add(page_b.clone());

        let removed = index.remove_earliest().unwrap();

        assert_eq!(2, removed.len());
        assert!(removed.iter().any(|p| p.page_id == page_a.page_id));
        assert!(removed.iter().any(|p| p.page_id == page_b.page_id));
        assert!(index.data.is_empty());
    }
}
