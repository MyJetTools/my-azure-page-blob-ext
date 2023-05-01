use my_azure_storage_sdk::page_blob::consts::BLOB_PAGE_SIZE;

pub struct MissingInterval {
    pub from_page_no: usize,
    pub amount: usize,
}

pub struct FoundPages<'s> {
    pages: Vec<Option<&'s [u8]>>,
    start_page_no: usize,
    pub missing_intervals: Vec<MissingInterval>,
}

impl<'s> FoundPages<'s> {
    pub fn new(start_page_no: usize, max_pages_amount: usize) -> Self {
        Self {
            pages: Vec::with_capacity(max_pages_amount),
            start_page_no,
            missing_intervals: Vec::new(),
        }
    }

    fn add_missing_interval(&mut self) {
        let last_missing_interval = self.missing_intervals.last_mut();
        if let Some(last_missing_interval) = last_missing_interval {
            if last_missing_interval.from_page_no + last_missing_interval.amount
                == self.start_page_no + self.pages.len()
            {
                last_missing_interval.amount += 1;
                return;
            }
        }

        self.missing_intervals.push(MissingInterval {
            from_page_no: self.start_page_no + self.pages.len(),
            amount: 1,
        });
    }

    pub fn add(&mut self, page: Option<&'s [u8]>) {
        if page.is_none() {
            self.add_missing_interval()
        }

        self.pages.push(page);
    }

    pub fn upload_missing_pages(&mut self, payload: &'s [u8]) {
        for missing_interval in &self.missing_intervals {
            for i in 0..missing_interval.amount {
                let offset = i * BLOB_PAGE_SIZE;

                self.pages[missing_interval.from_page_no - self.start_page_no + i] =
                    Some(&payload[offset..offset + BLOB_PAGE_SIZE]);
            }
        }
    }

    pub fn get_pages_to_upload(&self) -> Option<MissingInterval> {
        let start_page = self.missing_intervals.get(0)?.from_page_no;

        let last_missing_interval = self.missing_intervals.last()?;

        let last_page = last_missing_interval.from_page_no + last_missing_interval.amount;

        MissingInterval {
            from_page_no: start_page,
            amount: last_page - start_page,
        }
        .into()
    }

    pub fn into_vec(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.pages.len() * BLOB_PAGE_SIZE);

        for item in &self.pages {
            result.extend_from_slice(item.unwrap());
        }

        result
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_no_missing_intervals() {
        let test_page = vec![0u8; 512];
        let mut found_pages = FoundPages::new(0, 3);

        found_pages.add(Some(&test_page));
        found_pages.add(Some(&test_page));
        found_pages.add(Some(&test_page));

        assert_eq!(found_pages.missing_intervals.len(), 0);
        assert!(found_pages.get_pages_to_upload().is_none())
    }

    #[test]
    fn test_one_missing_intervals() {
        let test_page = vec![0u8; 512];
        let mut found_pages = FoundPages::new(10, 3);

        found_pages.add(Some(&test_page));
        found_pages.add(None);
        found_pages.add(Some(&test_page));

        assert_eq!(found_pages.missing_intervals.len(), 1);

        assert_eq!(found_pages.missing_intervals[0].from_page_no, 11);
        assert_eq!(found_pages.missing_intervals[0].amount, 1);

        let pages_to_upload = found_pages.get_pages_to_upload().unwrap();
        assert_eq!(pages_to_upload.from_page_no, 11);
        assert_eq!(pages_to_upload.amount, 1);
    }

    #[test]
    fn test_one_missing_intervals_with_two_pages() {
        let test_page = vec![0u8; 512];
        let mut found_pages = FoundPages::new(10, 4);

        found_pages.add(Some(&test_page));
        found_pages.add(None);
        found_pages.add(None);
        found_pages.add(Some(&test_page));

        assert_eq!(found_pages.missing_intervals.len(), 1);

        assert_eq!(found_pages.missing_intervals[0].from_page_no, 11);
        assert_eq!(found_pages.missing_intervals[0].amount, 2);

        let pages_to_upload = found_pages.get_pages_to_upload().unwrap();
        assert_eq!(pages_to_upload.from_page_no, 11);
        assert_eq!(pages_to_upload.amount, 2);
    }

    #[test]
    fn test_two_missing_intervals() {
        let test_page = vec![0u8; 512];
        let mut found_pages = FoundPages::new(10, 8);

        found_pages.add(Some(&test_page));
        found_pages.add(None);
        found_pages.add(None);
        found_pages.add(Some(&test_page));
        found_pages.add(None);
        found_pages.add(None);
        found_pages.add(None);
        found_pages.add(Some(&test_page));

        assert_eq!(found_pages.missing_intervals.len(), 2);

        assert_eq!(found_pages.missing_intervals[0].from_page_no, 11);
        assert_eq!(found_pages.missing_intervals[0].amount, 2);

        assert_eq!(found_pages.missing_intervals[1].from_page_no, 14);
        assert_eq!(found_pages.missing_intervals[1].amount, 3);

        let pages_to_upload = found_pages.get_pages_to_upload().unwrap();
        assert_eq!(pages_to_upload.from_page_no, 11);
        assert_eq!(pages_to_upload.amount, 6);
    }

    #[test]
    fn test_adding_missing_part_which_fills_two_pages_and_getting_result() {
        let mut found_pages = FoundPages::new(10, 4);

        let test_page = vec![0u8; 512];
        found_pages.add(Some(&test_page));
        found_pages.add(None);

        let test_page_2 = vec![2u8; 512];
        found_pages.add(Some(&test_page_2));

        let missing_interval = vec![1u8; 512];
        found_pages.upload_missing_pages(&missing_interval);

        let mut result = vec![0u8; 512];
        result.extend_from_slice([1u8; 512].as_slice());
        result.extend_from_slice([2u8; 512].as_slice());

        assert_eq!(result.as_slice(), found_pages.into_vec().as_slice());
    }

    #[test]
    fn test_adding_missing_parts_and_getting_result() {
        let mut found_pages = FoundPages::new(10, 4);

        let test_page = vec![0u8; 512];
        found_pages.add(Some(&test_page));
        found_pages.add(None);
        found_pages.add(None);

        let test_page = vec![1u8; 512];
        found_pages.add(Some(&test_page));
        found_pages.add(None);
        found_pages.add(None);
        found_pages.add(None);

        let test_page = vec![2u8; 512];
        found_pages.add(Some(&test_page));

        let pages_to_upload = found_pages.get_pages_to_upload().unwrap();

        let mut data_to_upload = Vec::with_capacity(pages_to_upload.amount * BLOB_PAGE_SIZE);

        data_to_upload.resize(pages_to_upload.amount * BLOB_PAGE_SIZE, 3u8);

        found_pages.upload_missing_pages(data_to_upload.as_slice());

        let result = found_pages.into_vec();

        let mut expected_result = vec![0u8; 512];
        expected_result.extend_from_slice([3u8; 512 * 2].as_slice());
        expected_result.extend_from_slice([1u8; 512].as_slice());
        expected_result.extend_from_slice([3u8; 512 * 3].as_slice());
        expected_result.extend_from_slice([2u8; 512].as_slice());

        assert_eq!(expected_result.as_slice(), result.as_slice());
    }
}
