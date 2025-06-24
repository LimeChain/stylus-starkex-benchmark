use crate::consts::input_offsets::OFFSET_PUBLIC_MEMORY;
use crate::consts::page_info::{PAGE_INFO_SIZE, PAGE_INFO_SIZE_OFFSET};

pub fn offset_page_size(page_index: usize) -> usize {
    OFFSET_PUBLIC_MEMORY + PAGE_INFO_SIZE * page_index + PAGE_INFO_SIZE_OFFSET
}

pub fn public_input_length(n_pages: usize) -> usize {
    OFFSET_PUBLIC_MEMORY + (PAGE_INFO_SIZE + 1) * n_pages
}
