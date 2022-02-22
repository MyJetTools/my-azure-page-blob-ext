pub fn get_pages_amount_by_size(data_size: usize, page_size: usize) -> usize {
    (data_size - 1) / page_size + 1
}

pub fn calc_pages_amount_to_ressize(position: usize, page_size: usize, pages_rate: usize) -> usize {
    let big_page_size = page_size * pages_rate;
    let big_pages_amount = crate::utils::get_pages_amount_by_size(position, big_page_size);
    big_pages_amount * pages_rate
}
