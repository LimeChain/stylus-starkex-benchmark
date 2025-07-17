//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256},
    console,
    prelude::*,
};

sol_storage! {
    #[entrypoint]
    pub struct MockProvider {
        uint256 number;
    }
}

#[public]
impl MockProvider {
    pub fn is_valid(&self, fact: FixedBytes<32>) -> Result<bool, Vec<u8>> {
        Ok(true)
    }
}
