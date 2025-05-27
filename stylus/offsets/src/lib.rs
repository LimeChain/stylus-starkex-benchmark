pub mod page_info {
    pub const PAGE_INFO_SIZE: usize = 3;
    pub const PAGE_INFO_SIZE_IN_BYTES: usize = 3 * 32;
    pub const PAGE_INFO_ADDRESS_OFFSET: usize = 0;
    pub const PAGE_INFO_SIZE_OFFSET: usize = 1;
    pub const PAGE_INFO_HASH_OFFSET: usize = 2;
    // A regular page entry is a (address, value) pair stored as 2 uint256 words.
    pub const MEMORY_PAIR_SIZE: usize = 2;
}

pub mod public_input_offsets {
    // The following constants are offsets of data expected in the public input.
    pub const OFFSET_N_VERIFIER_FRIENDLY_LAYERS: usize = 0;
    pub const OFFSET_LOG_N_STEPS: usize = 1;
    pub const OFFSET_RC_MIN: usize = 2;
    pub const OFFSET_RC_MAX: usize = 3;
    pub const OFFSET_LAYOUT_CODE: usize = 4;
    pub const OFFSET_PROGRAM_BEGIN_ADDR: usize = 5;
    pub const OFFSET_PROGRAM_STOP_PTR: usize = 6;
    pub const OFFSET_EXECUTION_BEGIN_ADDR: usize = 7;
    pub const OFFSET_EXECUTION_STOP_PTR: usize = 8;
    pub const OFFSET_OUTPUT_BEGIN_ADDR: usize = 9;
    pub const OFFSET_OUTPUT_STOP_PTR: usize = 10;
    pub const OFFSET_PEDERSEN_BEGIN_ADDR: usize = 11;
    pub const OFFSET_PEDERSEN_STOP_PTR: usize = 12;
    pub const OFFSET_RANGE_CHECK_BEGIN_ADDR: usize = 13;
    pub const OFFSET_RANGE_CHECK_STOP_PTR: usize = 14;
    pub const OFFSET_BITWISE_BEGIN_ADDR: usize = 15;
    pub const OFFSET_BITWISE_STOP_PTR: usize = 16;
    pub const OFFSET_POSEIDON_BEGIN_ADDR: usize = 17;
    pub const OFFSET_POSEIDON_STOP_PTR: usize = 18;
    pub const OFFSET_PUBLIC_MEMORY_PADDING_ADDR: usize = 19;
    pub const OFFSET_PUBLIC_MEMORY_PADDING_VALUE: usize = 20;
    pub const OFFSET_N_PUBLIC_MEMORY_PAGES: usize = 21;
    pub const OFFSET_PUBLIC_MEMORY: usize = 22;

    // The program segment starts from 1, so that memory address 0 is kept for the null pointer.
    pub const INITIAL_PC: usize = 1;

    pub const FINAL_PC: usize = INITIAL_PC + 4;
}
