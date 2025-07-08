pub mod mmaps {
    pub const MM_N_UNIQUE_QUERIES: usize = 9;
    pub const MM_FRI_QUEUE_START: usize = 109;
    pub const MM_TRACE_GENERATOR: usize = 350;
    pub const MM_OODS_VALUES_START: usize = 359;
    pub const MM_COMPOSITION_OODS_VALUES_START: usize = 551;
    pub const MM_POINT: usize = 351;
    pub const MM_EVAL_POINTS_START: usize = 553;
    pub const MM_OODS_ALPHA: usize = 601;
    pub const MM_TRACE_QUERY_RESPONSES_START: usize = 602;

    pub const MM_COMPOSITION_QUERY_RESPONSES_START: usize = 1178;
}

pub mod stark_params {
    pub const N_ROWS_IN_MASK: usize = 98;
}

pub mod prime_field_element0 {
    use stylus_sdk::alloy_primitives::{uint, U256};

    pub const GENERATOR_VAL: U256 = uint!(3_U256);
    pub const K_MONTGOMERY_R_INV: U256 =
        uint!(0x40000000000001100000000000012100000000000000000000000000000000_U256);
}