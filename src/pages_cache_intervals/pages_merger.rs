use my_azure_storage_sdk::page_blob::consts::BLOB_PAGE_SIZE;

use crate::PagesCacheItem;

pub fn merge_pages<'s>(
    src_pages: &[&'s PagesCacheItem],
    page_to_merge: &PagesCacheItem,
) -> PagesCacheItem {
    let (min_page_id, max_pages_id) = get_min_max_page_id(src_pages, page_to_merge);

    let mut result_content: Vec<u8> =
        Vec::with_capacity((max_pages_id - min_page_id) * BLOB_PAGE_SIZE);

    for page_id in min_page_id..max_pages_id {
        push_content(&mut result_content, src_pages, page_to_merge, page_id);
    }

    PagesCacheItem::new(min_page_id, result_content)
}

pub fn get_min_max_page_id<'s>(
    src_pages: &[&'s PagesCacheItem],
    page_to_merge: &PagesCacheItem,
) -> (usize, usize) {
    let mut min_page_id = page_to_merge.page_id;

    let mut max_pages_id = page_to_merge.page_id + page_to_merge.get_pages_amount();

    for page in src_pages {
        let max_page_no = page.get_pages_amount() + page.page_id;
        if max_page_no > max_pages_id {
            max_pages_id = max_page_no;
        }

        if min_page_id > page.page_id {
            min_page_id = page.page_id;
        }
    }

    (min_page_id, max_pages_id)
}

pub fn push_content<'s>(
    result_content: &mut Vec<u8>,
    src_pages: &[&'s PagesCacheItem],
    page_to_merge: &PagesCacheItem,
    page_id: usize,
) {
    if let Some(content) = page_to_merge.get_content(page_id) {
        result_content.extend_from_slice(content);
        return;
    }

    for src_page in src_pages {
        if let Some(content) = src_page.get_content(page_id) {
            result_content.extend_from_slice(content);
            return;
        }
    }

    result_content.extend_from_slice([0u8; BLOB_PAGE_SIZE].as_ref());
}

#[cfg(test)]
mod tests {

    use crate::PagesCacheItem;

    #[test]
    fn test_merge_pages_with_white_spaces() {
        let src_pages = vec![
            PagesCacheItem::new(0, vec![0u8; 512]),
            PagesCacheItem::new(2, vec![2u8; 512]),
        ];

        let src_pages: Vec<&PagesCacheItem> = src_pages.iter().collect();

        let page_to_merge = PagesCacheItem::new(1, vec![1u8; 512]);

        let result_page = super::merge_pages(src_pages.as_slice(), &page_to_merge);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 512].as_ref());
        result.extend_from_slice([2u8; 512].as_ref());

        assert_eq!(result_page.content, result);
    }

    #[test]
    fn test_merge_pages_with_overlap() {
        let src_pages = vec![
            PagesCacheItem::new(0, vec![0u8; 512]),
            PagesCacheItem::new(2, vec![2u8; 1024]),
        ];

        let src_pages: Vec<&PagesCacheItem> = src_pages.iter().collect();

        let page_to_merge = PagesCacheItem::new(1, vec![1u8; 1024]);

        let result_page = super::merge_pages(src_pages.as_slice(), &page_to_merge);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 1024].as_ref());
        result.extend_from_slice([2u8; 512].as_ref());

        assert_eq!(result_page.content, result);
    }
}
