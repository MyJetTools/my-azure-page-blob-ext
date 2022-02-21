pub struct PagesInterval {
    pub start_page_no: usize,
    pub pages_amount: usize,
}

impl PagesInterval {
    pub fn new(value: usize) -> Self {
        Self {
            start_page_no: value,
            pages_amount: 1,
        }
    }
}
