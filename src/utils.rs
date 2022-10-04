pub fn get_pages_amount_by_size(data_size: usize, page_size: usize) -> usize {
    (data_size - 1) / page_size + 1
}

pub fn calc_pages_amount_to_ressize(position: usize, page_size: usize, pages_rate: usize) -> usize {
    let big_page_size = page_size * pages_rate;
    let big_pages_amount = crate::utils::get_pages_amount_by_size(position, big_page_size);
    big_pages_amount * pages_rate
}

pub fn fill_content_to_comply_with_page_blob_size(
    content: &mut Vec<u8>,
    byte_to_fill: u8,
    blob_page_size: usize,
) {
    if content.len() == 0 {
        return;
    }

    let pages_amount = get_pages_amount_by_size(content.len(), blob_page_size);

    let required_size = pages_amount * blob_page_size;

    while content.len() < required_size {
        content.push(byte_to_fill);
    }
}

#[cfg(test)]
mod tests {
    use my_azure_storage_sdk::page_blob::consts::BLOB_PAGE_SIZE;

    use super::fill_content_to_comply_with_page_blob_size;

    #[test]
    fn test_page_blob_resize() {
        let mut content = vec![1u8, 2u8, 3u8];

        fill_content_to_comply_with_page_blob_size(&mut content, 32u8, BLOB_PAGE_SIZE);

        assert_eq!(BLOB_PAGE_SIZE, content.len());
    }

    #[test]
    fn test_page_blob_resize_zero_content() {
        let mut content = vec![];

        fill_content_to_comply_with_page_blob_size(&mut content, 32u8, BLOB_PAGE_SIZE);

        assert_eq!(0, content.len());
    }

    #[test]
    fn test_page_blob_resize_if_we_have_exactly_one_page() {
        let mut content = vec![0u8; 512];

        fill_content_to_comply_with_page_blob_size(&mut content, 32u8, BLOB_PAGE_SIZE);

        assert_eq!(BLOB_PAGE_SIZE, content.len());
    }
}
