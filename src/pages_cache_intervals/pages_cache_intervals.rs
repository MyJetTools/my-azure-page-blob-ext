use crate::PagesCacheItem;

enum InsertToPagesAction {
    AtIndex(usize),
    AtTheEnd,
    MergeWithPage(usize),
    MergeWithPages(Vec<usize>),
}

pub struct PagesCacheIntervals {
    pub pages: Vec<PagesCacheItem>,
}

impl PagesCacheIntervals {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn update_pages(&mut self, page_no: usize, content: Vec<u8>) {
        let new_item = PagesCacheItem::new(page_no, content);
        let action = self.get_page_index_to_merge(&new_item);

        match action {
            InsertToPagesAction::AtIndex(page_index) => {
                self.pages.insert(page_index, new_item);
            }
            InsertToPagesAction::AtTheEnd => {
                self.pages.push(new_item);
            }
            InsertToPagesAction::MergeWithPage(page_index) => {
                self.pages.get_mut(page_index).unwrap().merge(new_item);
            }
            InsertToPagesAction::MergeWithPages(pages) => {
                let pages_to_merge: Vec<&PagesCacheItem> = pages
                    .iter()
                    .map(|page_index| self.pages.get(*page_index).unwrap())
                    .collect();

                let merged_page =
                    super::pages_merger::merge_pages(pages_to_merge.as_slice(), &new_item);

                let start_index = *pages.get(0).unwrap();

                self.remove_pages(start_index, pages.len());

                self.pages.insert(start_index, merged_page);
            }
        }
    }

    fn remove_pages(&mut self, at_index: usize, amount: usize) {
        self.pages.drain(at_index..at_index + amount);
    }
    fn get_page_index_to_merge(&self, new_content: &PagesCacheItem) -> InsertToPagesAction {
        let mut no = 0;

        let mut found_pages = Vec::with_capacity(self.pages.len());

        for page in &self.pages {
            match page.is_my_page_to_merge(&new_content) {
                crate::IsMyPageToAddResult::Before => {
                    if found_pages.len() == 0 {
                        return InsertToPagesAction::AtIndex(no);
                    }
                    break;
                }
                crate::IsMyPageToAddResult::Here => {
                    found_pages.push(no);
                }
                crate::IsMyPageToAddResult::After => {}
            }
            no += 1;
        }

        match found_pages.len() {
            0 => {
                return InsertToPagesAction::AtTheEnd;
            }
            1 => {
                return InsertToPagesAction::MergeWithPage(*found_pages.get(0).unwrap());
            }
            _ => {
                return InsertToPagesAction::MergeWithPages(found_pages);
            }
        }
    }

    pub fn get_page(&self, page_id: usize) -> Option<&[u8]> {
        for page in &self.pages {
            if let Some(page_content) = page.get_content(page_id) {
                return Some(page_content);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_we_merge_page_exactly_before() {
        let mut pages_cache = PagesCacheIntervals::new();

        pages_cache.update_pages(2, vec![0u8; 512]);

        assert_eq!(1, pages_cache.pages.len());

        assert_eq!([0u8; 512].as_slice(), pages_cache.pages[0].content);
        assert_eq!(2, pages_cache.pages[0].page_id);

        pages_cache.update_pages(1, vec![1u8; 512]);

        assert_eq!(1, pages_cache.pages.len());

        let mut result = vec![1u8; 512];
        result.extend_from_slice([0u8; 512].as_slice());

        assert_eq!(result, pages_cache.pages[0].content);
        assert_eq!(1, pages_cache.pages[0].page_id);
    }

    #[test]
    fn test_we_merge_page_exactly_after() {
        let mut pages_cache = PagesCacheIntervals::new();

        pages_cache.update_pages(1, vec![0u8; 512]);

        assert_eq!(1, pages_cache.pages.len());

        assert_eq!([0u8; 512].as_slice(), pages_cache.pages[0].content);
        assert_eq!(1, pages_cache.pages[0].page_id);

        pages_cache.update_pages(2, vec![1u8; 512]);

        assert_eq!(1, pages_cache.pages.len());

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 512].as_slice());

        assert_eq!(result, pages_cache.pages[0].content);
        assert_eq!(1, pages_cache.pages[0].page_id);
    }

    #[test]
    fn test_we_insert_page_inside() {
        let mut pages_cache = PagesCacheIntervals::new();

        pages_cache.update_pages(1, vec![0u8; 512]);
        pages_cache.update_pages(3, vec![2u8; 512]);

        assert_eq!(2, pages_cache.pages.len());

        pages_cache.update_pages(2, vec![1u8; 512]);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 512].as_slice());
        result.extend_from_slice([2u8; 512].as_slice());

        assert_eq!(1, pages_cache.pages.len());
        assert_eq!(1, pages_cache.pages[0].page_id);
        assert_eq!(result, pages_cache.pages[0].content);
    }

    #[test]
    fn test_we_insert_page_inside_with_overlap_before() {
        let mut pages_cache = PagesCacheIntervals::new();

        pages_cache.update_pages(1, vec![0u8; 1024]);
        pages_cache.update_pages(4, vec![2u8; 512]);

        assert_eq!(2, pages_cache.pages.len());

        pages_cache.update_pages(2, vec![1u8; 1024]);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 1024].as_slice());
        result.extend_from_slice([2u8; 512].as_slice());

        assert_eq!(1, pages_cache.pages.len());
        assert_eq!(1, pages_cache.pages[0].page_id);
        assert_eq!(result, pages_cache.pages[0].content);
    }

    #[test]
    fn test_we_insert_page_inside_with_overlap_both_sides() {
        let mut pages_cache = PagesCacheIntervals::new();

        pages_cache.update_pages(1, vec![0u8; 1024]);
        pages_cache.update_pages(4, vec![2u8; 1024]);

        assert_eq!(2, pages_cache.pages.len());

        pages_cache.update_pages(2, vec![1u8; 512 * 3]);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 512 * 3].as_slice());
        result.extend_from_slice([2u8; 512].as_slice());

        assert_eq!(1, pages_cache.pages.len());
        assert_eq!(1, pages_cache.pages[0].page_id);
        assert_eq!(result, pages_cache.pages[0].content);
    }

    #[test]
    fn test_we_insert_page_inside_with_overlap_both_sides_and_replace_in_the_middle() {
        let mut pages_cache = PagesCacheIntervals::new();

        pages_cache.update_pages(1, vec![1u8; 1024]);
        pages_cache.update_pages(4, vec![2u8; 1024]);
        pages_cache.update_pages(7, vec![3u8; 1024]);

        assert_eq!(3, pages_cache.pages.len());

        pages_cache.update_pages(2, vec![4u8; 512 * 6]);

        let mut result = vec![1u8; 512];
        result.extend_from_slice([4u8; 512 * 6].as_slice());
        result.extend_from_slice([3u8; 512].as_slice());

        assert_eq!(1, pages_cache.pages.len());
        assert_eq!(1, pages_cache.pages[0].page_id);
        assert_eq!(result, pages_cache.pages[0].content);
    }

    #[test]
    fn test_we_insert_page_inside_with_no_connection() {
        let mut pages_cache = PagesCacheIntervals::new();

        pages_cache.update_pages(1, vec![0u8; 512]);
        pages_cache.update_pages(5, vec![2u8; 512]);

        assert_eq!(2, pages_cache.pages.len());

        pages_cache.update_pages(3, vec![1u8; 512]);

        assert_eq!(3, pages_cache.pages.len());

        assert_eq!(1, pages_cache.pages[0].page_id);
        assert_eq!(3, pages_cache.pages[1].page_id);
        assert_eq!(5, pages_cache.pages[2].page_id);
    }

    #[test]
    fn test_get_content() {
        let mut pages_cache = PagesCacheIntervals::new();
        pages_cache.update_pages(1, vec![1u8; 1024]);
        pages_cache.update_pages(5, vec![2u8; 512]);

        assert!(pages_cache.get_page(0).is_none());
        assert_eq!(pages_cache.get_page(1).unwrap(), [1u8; 512].as_slice());
        assert_eq!(pages_cache.get_page(2).unwrap(), [1u8; 512].as_slice());
        assert!(pages_cache.get_page(3).is_none());
        assert!(pages_cache.get_page(4).is_none());
        assert_eq!(pages_cache.get_page(5).unwrap(), [2u8; 512].as_slice());
        assert!(pages_cache.get_page(6).is_none());
    }
}
