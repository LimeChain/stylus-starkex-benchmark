pub mod public_input_offsets {
    pub const INITIAL_PC: usize = 1;
    pub const OFFSET_EXECUTION_BEGIN_ADDR: usize = 7;
    pub const OFFSET_EXECUTION_STOP_PTR: usize = 8;
    pub const OFFSET_OUTPUT_BEGIN_ADDR: usize = 9;
    pub const OFFSET_OUTPUT_STOP_PTR: usize = 10;
}

pub mod page_info {
    use stylus_sdk::alloy_primitives::{uint, U256};
    pub const PAGE_INFO_SIZE: usize = 3;

    // pub const PAGE_INFO_SIZE_IN_BYTES: usize = PAGE_INFO_SIZE * 32;

    // pub const PAGE_INFO_ADDRESS_OFFSET: usize = 0;
    pub const PAGE_INFO_SIZE_OFFSET: usize = 1;
    pub const PAGE_INFO_HASH_OFFSET: usize = 2;

    // pub const MEMORY_PAIR_SIZE: usize = 2;

    pub const MAX_PAGE_SIZE: U256 = uint!(1073741824_U256); // 2^30
}
