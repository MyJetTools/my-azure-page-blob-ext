pub struct ExactPayload {
    pub payload: Vec<u8>,
    pub offset: usize,
    pub size: usize,
}

impl ExactPayload {
    pub fn as_slice(&self) -> &[u8] {
        &self.payload[self.offset..self.offset + self.size]
    }
}

impl AsRef<[u8]> for ExactPayload {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

pub struct ExactPayloadPositions {
    pub start_page_no: usize,
    pub start_pos: usize,
    pub end_pos: usize,
    pub payload_size: usize,

    page_size: usize,
}

impl ExactPayloadPositions {
    pub fn new(start_pos: usize, payload_size: usize, page_size: usize) -> Self {
        Self {
            start_page_no: start_pos / page_size,
            start_pos,
            payload_size,
            page_size,
            end_pos: start_pos + payload_size,
        }
    }

    pub fn get_payload_offset(&self) -> usize {
        self.start_pos - self.start_page_no * self.page_size
    }

    pub fn get_full_pages_amount(&self) -> usize {
        crate::utils::get_pages_amount_by_size(
            self.end_pos - self.start_page_no * self.page_size,
            self.page_size,
        )
    }
}

#[cfg(test)]

mod test {

    use super::*;

    #[test]
    fn test_everything_is_inside_first_page() {
        let positions = ExactPayloadPositions::new(3, 3, 8);

        assert_eq!(3, positions.start_pos);
        assert_eq!(6, positions.end_pos);
        assert_eq!(0, positions.start_page_no);
        assert_eq!(3, positions.payload_size);
        assert_eq!(1, positions.get_full_pages_amount());
        assert_eq!(3, positions.get_payload_offset());
    }

    #[test]
    fn test_everything_is_second_page() {
        let positions = ExactPayloadPositions::new(9, 3, 8);

        assert_eq!(9, positions.start_pos);
        assert_eq!(12, positions.end_pos);
        assert_eq!(1, positions.start_page_no);
        assert_eq!(3, positions.payload_size);
        assert_eq!(1, positions.get_full_pages_amount());
        assert_eq!(1, positions.get_payload_offset());
    }

    #[test]
    fn test_we_start_in_the_middle_of_second_page_end_in_the_middle_of_page_3() {
        let positions = ExactPayloadPositions::new(9, 10, 8);

        assert_eq!(9, positions.start_pos);
        assert_eq!(19, positions.end_pos);
        assert_eq!(1, positions.start_page_no);
        assert_eq!(10, positions.payload_size);
        assert_eq!(2, positions.get_full_pages_amount());
        assert_eq!(1, positions.get_payload_offset());
    }
}
