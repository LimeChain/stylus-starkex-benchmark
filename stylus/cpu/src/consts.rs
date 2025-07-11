// The following constants are offsets of data expected in the public input.
pub mod input_offsets {
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
}

pub mod page_info {
    pub const PAGE_INFO_SIZE: usize = 3;
    pub const PAGE_INFO_SIZE_OFFSET: usize = 1;
}

pub mod memory_offsets {
    pub const MAX_N_QUERIES: u32 = 48;
    pub const FRI_QUEUE_SIZE: u32 = MAX_N_QUERIES;
    pub const MAX_FRI_STEPS: usize = 10;
    pub const MAX_SUPPORTED_FRI_STEP_SIZE: u32 = 4;

    pub const MM_EVAL_DOMAIN_SIZE: u32 = 0x0;
    pub const MM_BLOW_UP_FACTOR: u32 = 0x1;
    pub const MM_LOG_EVAL_DOMAIN_SIZE: u32 = 0x2;
    pub const MM_PROOF_OF_WORK_BITS: u32 = 0x3;
    pub const MM_EVAL_DOMAIN_GENERATOR: u32 = 0x4;
    pub const MM_PUBLIC_INPUT_PTR: u32 = 0x5;
    pub const MM_TRACE_COMMITMENT: u32 = 0x6; // uint256[2]
    pub const MM_OODS_COMMITMENT: u32 = 0x8;
    pub const MM_N_UNIQUE_QUERIES: u32 = 0x9;
    pub const MM_CHANNEL: u32 = 0xa; // uint256[3]
    pub const MM_MERKLE_QUEUE: u32 = 0xd; // uint256[96]
    pub const MM_FRI_QUEUE: u32 = 0x6d; // uint256[144]
    pub const MM_FRI_QUERIES_DELIMITER: u32 = 0xfd;
    pub const MM_FRI_CTX: u32 = 0xfe; // uint256[40]
    pub const MM_FRI_STEP_SIZES_PTR: u32 = 0x126;
    pub const MM_FRI_EVAL_POINTS: u32 = 0x127; // uint256[10]
    pub const MM_FRI_COMMITMENTS: u32 = 0x131; // uint256[10]
    pub const MM_FRI_LAST_LAYER_DEG_BOUND: u32 = 0x13b;
    pub const MM_FRI_LAST_LAYER_PTR: u32 = 0x13c;
    pub const MM_CONSTRAINT_POLY_ARGS_START: u32 = 0x13d;
    pub const MM_PERIODIC_COLUMN__PEDERSEN__POINTS__X: u32 = 0x13d;
    pub const MM_PERIODIC_COLUMN__PEDERSEN__POINTS__Y: u32 = 0x13e;
    pub const MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY0: u32 = 0x13f;
    pub const MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY1: u32 = 0x140;
    pub const MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY2: u32 = 0x141;
    pub const MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__PARTIAL_ROUND_KEY0: u32 = 0x142;
    pub const MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__PARTIAL_ROUND_KEY1: u32 = 0x143;
    pub const MM_TRACE_LENGTH: u32 = 0x144;
    pub const MM_OFFSET_SIZE: usize = 0x145;
    pub const MM_HALF_OFFSET_SIZE: usize = 0x146;
    pub const MM_INITIAL_AP: usize = 0x147;
    pub const MM_INITIAL_PC: usize = 0x148;
    pub const MM_FINAL_AP: usize = 0x149;
    pub const MM_FINAL_PC: usize = 0x14a;
    pub const MM_MEMORY__MULTI_COLUMN_PERM__PERM__INTERACTION_ELM: u32 = 0x14b;
    pub const MM_MEMORY__MULTI_COLUMN_PERM__HASH_INTERACTION_ELM0: u32 = 0x14c;
    pub const MM_MEMORY__MULTI_COLUMN_PERM__PERM__PUBLIC_MEMORY_PROD: u32 = 0x14d;
    pub const MM_RANGE_CHECK16__PERM__INTERACTION_ELM: u32 = 0x14e;
    pub const MM_RANGE_CHECK16__PERM__PUBLIC_MEMORY_PROD: u32 = 0x14f;
    pub const MM_RANGE_CHECK_MIN: usize = 0x150;
    pub const MM_RANGE_CHECK_MAX: usize = 0x151;
    pub const MM_DILUTED_CHECK__PERMUTATION__INTERACTION_ELM: u32 = 0x152;
    pub const MM_DILUTED_CHECK__PERMUTATION__PUBLIC_MEMORY_PROD: u32 = 0x153;
    pub const MM_DILUTED_CHECK__FIRST_ELM: u32 = 0x154;
    pub const MM_DILUTED_CHECK__INTERACTION_Z: u32 = 0x155;
    pub const MM_DILUTED_CHECK__INTERACTION_ALPHA: u32 = 0x156;
    pub const MM_DILUTED_CHECK__FINAL_CUM_VAL: u32 = 0x157;
    pub const MM_PEDERSEN__SHIFT_POINT_X: u32 = 0x158;
    pub const MM_PEDERSEN__SHIFT_POINT_Y: u32 = 0x159;
    pub const MM_INITIAL_PEDERSEN_ADDR: usize = 0x15a;
    pub const MM_INITIAL_RANGE_CHECK_ADDR: u32 = 0x15b;
    pub const MM_INITIAL_BITWISE_ADDR: u32 = 0x15c;
    pub const MM_INITIAL_POSEIDON_ADDR: u32 = 0x15d;
    pub const MM_TRACE_GENERATOR: u32 = 0x15e;
    pub const MM_OODS_POINT: u32 = 0x15f;
    pub const MM_INTERACTION_ELEMENTS: u32 = 0x160; // uint256[6]
    pub const MM_COMPOSITION_ALPHA: u32 = 0x166;
    pub const MM_OODS_VALUES: u32 = 0x167; // uint256[192]
    pub const MM_CONSTRAINT_POLY_ARGS_END: u32 = 0x227;
    pub const MM_COMPOSITION_OODS_VALUES: u32 = 0x227; // uint256[2]
    pub const MM_OODS_EVAL_POINTS: u32 = 0x229; // uint256[48]
    pub const MM_OODS_ALPHA: u32 = 0x259;
    pub const MM_TRACE_QUERY_RESPONSES: u32 = 0x25a; // uint256[576]
    pub const MM_COMPOSITION_QUERY_RESPONSES: u32 = 0x49a; // uint256[96]
    pub const MM_LOG_N_STEPS: usize = 0x4fa;
    pub const MM_N_PUBLIC_MEM_ENTRIES: usize = 0x4fb;
    pub const MM_N_PUBLIC_MEM_PAGES: usize = 0x4fc;
    pub const MM_CONTEXT_SIZE: usize = 0x4fd;
}

pub mod stark_parameters {
    pub const PEDERSEN_BUILTIN_RATIO: usize = 128;
    pub const LOG_CPU_COMPONENT_HEIGHT: usize = 4;
}
