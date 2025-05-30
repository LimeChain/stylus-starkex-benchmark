//! Stylus Cpu Verifier
//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;
mod errors;
mod stark_verifier;

use offsets::{page_info, public_input_offsets, PublicMemoryOffset};

use stark_verifier::*;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, prelude::*};

// #[storage]
// pub struct LayoutSpecific {}

#[storage]
pub struct CpuVerifier {}
impl PublicMemoryOffset for CpuVerifier {
    fn get_public_memory_offset() -> usize {
        public_input_offsets::OFFSET_PUBLIC_MEMORY
    }
}

#[public]
impl CpuVerifier {
    pub fn get_layout_info(&self) -> Result<(U256, U256), Vec<u8>> {
        let public_memory_offset: U256 =
            U256::from(public_input_offsets::OFFSET_N_PUBLIC_MEMORY_PAGES);
        let selected_builtins: U256 = U256::from(
            (1 << OUTPUT_BUILTIN_BIT)
                | (1 << PEDERSEN_BUILTIN_BIT)
                | (1 << RANGE_CHECK_BUILTIN_BIT)
                | (1 << ECDSA_BUILTIN_BIT)
                | (1 << BITWISE_BUILTIN_BIT)
                | (1 << EC_OP_BUILTIN_BIT),
        );
        Ok((public_memory_offset, selected_builtins))
    }
}

pub const OUTPUT_BUILTIN_BIT: usize = 0;
pub const PEDERSEN_BUILTIN_BIT: usize = 1;
pub const RANGE_CHECK_BUILTIN_BIT: usize = 2;
pub const ECDSA_BUILTIN_BIT: usize = 3;
pub const BITWISE_BUILTIN_BIT: usize = 4;
pub const EC_OP_BUILTIN_BIT: usize = 5;
pub const KECCAK_BUILTIN_BIT: usize = 6;
pub const POSEIDON_BUILTIN_BIT: usize = 7;
