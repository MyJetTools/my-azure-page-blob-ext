use my_azure_storage_sdk::page_blob::consts::BLOB_PAGE_SIZE;

pub enum IsMyPageToAddResult {
    Before,
    Here,
    After,
}

impl IsMyPageToAddResult {
    pub fn is_before(&self) -> bool {
        match self {
            IsMyPageToAddResult::Before => true,
            _ => false,
        }
    }

    pub fn is_here(&self) -> bool {
        match self {
            IsMyPageToAddResult::Here => true,
            _ => false,
        }
    }

    pub fn is_after(&self) -> bool {
        match self {
            IsMyPageToAddResult::After => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct PagesCacheItem {
    pub content: Vec<u8>,
    pub page_id: usize,
}

impl PagesCacheItem {
    pub fn new(page_id: usize, content: Vec<u8>) -> Self {
        Self { page_id, content }
    }

    pub fn get_pages_amount(&self) -> usize {
        self.content.len() / BLOB_PAGE_SIZE
    }

    pub fn get_last_page_id(&self) -> usize {
        self.page_id + self.get_pages_amount()
    }

    pub fn is_my_page_to_merge(&self, new_page: &Self) -> IsMyPageToAddResult {
        let last_page_id = new_page.get_last_page_id();
        if last_page_id < self.page_id {
            return IsMyPageToAddResult::Before;
        }

        if new_page.page_id > self.page_id + self.get_pages_amount() {
            return IsMyPageToAddResult::After;
        }

        IsMyPageToAddResult::Here
    }

    pub fn get_content(&self, page_id: usize) -> Option<&[u8]> {
        if page_id >= self.page_id && page_id < self.page_id + self.get_pages_amount() {
            let offset = (page_id - self.page_id) * BLOB_PAGE_SIZE;
            return Some(&self.content[offset..offset + BLOB_PAGE_SIZE]);
        }

        None
    }

    pub fn merge(&mut self, mut new_item: Self) {
        if new_item.get_last_page_id() == self.page_id {
            new_item.content.extend_from_slice(&self.content);
            self.content = new_item.content;
            self.page_id = new_item.page_id;
            return;
        }

        if new_item.page_id == self.page_id {
            if new_item.content.len() >= self.content.len() {
                self.content = new_item.content;
            } else {
                self.content[..new_item.content.len()]
                    .copy_from_slice(&new_item.content[..new_item.content.len()]);
            }

            return;
        }

        if new_item.page_id == self.get_last_page_id() {
            self.content.extend(new_item.content);
            return;
        }

        let truncate_to = new_item.page_id - self.page_id;
        let truncate_to = truncate_to * BLOB_PAGE_SIZE;
        self.content.resize(truncate_to, 0u8);
        self.content.extend(new_item.content);
    }
}

#[cfg(test)]
mod tests {

    use crate::PagesCacheItem;

    //Testing that page is selected as page to merge for single page data
    //        +
    // [0][1][2][3][4]
    // [B][H][H][H][A]
    #[test]
    fn test_my_page_to_merge() {
        let my_page = PagesCacheItem::new(2, vec![0u8; 512]);

        let mut new_page = PagesCacheItem::new(0, vec![0u8; 512]);
        assert!(my_page.is_my_page_to_merge(&new_page).is_before());

        new_page.page_id = 1;
        assert!(my_page.is_my_page_to_merge(&new_page).is_here());

        new_page.page_id = 2;
        assert!(my_page.is_my_page_to_merge(&new_page).is_here());

        new_page.page_id = 3;
        assert!(my_page.is_my_page_to_merge(&new_page).is_here());

        new_page.page_id = 4;
        assert!(my_page.is_my_page_to_merge(&new_page).is_after());
    }

    //Testing that page is selected as page to merge for single page data
    //                 +  +
    // [0][1][2][3][4][5][6][7][8]
    // [B][B][B][H][H][H][H][H][A]

    #[test]
    fn test_we_have_two_pages_to_merge_with_to_pages_() {
        let my_page = PagesCacheItem::new(5, vec![0u8; 1024]);

        let mut new_page = PagesCacheItem::new(2, vec![0u8; 512 * 2]);
        assert!(my_page.is_my_page_to_merge(&new_page).is_before());

        new_page.page_id = 3;

        assert!(my_page.is_my_page_to_merge(&new_page).is_here());

        new_page.page_id = 4;
        assert!(my_page.is_my_page_to_merge(&new_page).is_here());

        new_page.page_id = 5;
        assert!(my_page.is_my_page_to_merge(&new_page).is_here());

        new_page.page_id = 6;
        assert!(my_page.is_my_page_to_merge(&new_page).is_here());

        new_page.page_id = 7;
        assert!(my_page.is_my_page_to_merge(&new_page).is_here());

        let new_page = PagesCacheItem::new(8, vec![0u8; 512 * 5]);
        assert!(my_page.is_my_page_to_merge(&new_page).is_after());
    }

    #[test]
    fn test_merge_in_front_of_page() {
        let mut my_page = PagesCacheItem::new(1, vec![1u8; 512]);

        let new_content = PagesCacheItem::new(0, vec![0u8; 512]);

        my_page.merge(new_content);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 512].as_ref());

        assert_eq!(my_page.content, result);
    }

    #[test]
    fn test_merge_from_begin_half_of_a_page() {
        let mut my_page = PagesCacheItem::new(1, vec![0u8; 1024]);

        let new_content = PagesCacheItem::new(1, vec![1u8; 512]);

        my_page.merge(new_content);

        let mut result = vec![1u8; 512];
        result.extend_from_slice([0u8; 512].as_ref());

        assert_eq!(my_page.content, result);
    }

    #[test]
    fn test_merge_from_begin_same_size() {
        let mut my_page = PagesCacheItem::new(1, vec![0u8; 512]);

        let new_content = PagesCacheItem::new(1, vec![1u8; 512]);

        my_page.merge(new_content);

        let result = vec![1u8; 512];

        assert_eq!(my_page.content, result);
    }

    #[test]
    fn test_merge_from_begin_bigger_size() {
        let mut my_page = PagesCacheItem::new(1, vec![0u8; 512]);

        let new_content = PagesCacheItem::new(1, vec![1u8; 1024]);

        my_page.merge(new_content);

        let result = vec![1u8; 1024];

        assert_eq!(my_page.content, result);
    }

    #[test]
    fn test_merge_at_the_end() {
        let mut my_page = PagesCacheItem::new(1, vec![0u8; 512]);

        let new_content = PagesCacheItem::new(2, vec![1u8; 512]);

        my_page.merge(new_content);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 512].as_slice());

        assert_eq!(my_page.content, result);
    }

    #[test]
    fn test_merge_at_in_the_middle() {
        let mut my_page = PagesCacheItem::new(1, vec![0u8; 1024]);

        let new_content = PagesCacheItem::new(2, vec![1u8; 512]);

        my_page.merge(new_content);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 512].as_slice());

        assert_eq!(my_page.content, result);
    }

    #[test]
    fn test_merge_at_in_the_middle_and_overflow() {
        let mut my_page = PagesCacheItem::new(1, vec![0u8; 1024]);

        let new_content = PagesCacheItem::new(2, vec![1u8; 1024]);

        my_page.merge(new_content);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 1024].as_slice());

        assert_eq!(my_page.content, result);
    }
}
