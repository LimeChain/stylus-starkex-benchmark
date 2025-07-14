pub struct PublicMemoryOffset {}

impl PublicMemoryOffset {

    pub fn get_offset_page_size(page_id: usize) -> usize {
        22 + 3 * page_id - 1 + 1
    }

    pub fn get_offset_page_hash(page_id: usize) -> usize {
        22 + 3 * page_id - 1 + 2
    }

    pub fn get_offset_page_addr(page_id: usize) -> usize {
        22 + 3 * page_id - 1
    }

    pub fn get_offset_page_prod(page_id: usize, n_pages: usize) -> usize {
        22 + 3 * n_pages - 1 + page_id
    }

    pub fn get_public_input_length(n_pages: usize) -> usize {
        22 + (3 + 1) * n_pages - 1
    }
}
