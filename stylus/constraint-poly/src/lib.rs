// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::alloy_primitives::{address, uint, Address, U256};

use stylus_sdk::call::{static_call, Call};
use stylus_sdk::console;
use stylus_sdk::{prelude::*, storage::StorageAddress, ArbResult};

// debug imports

#[storage]
#[entrypoint]
pub struct ConstraintPoly {
    preparer_address: StorageAddress,
    finalizer_address: StorageAddress,
}

#[public]
impl ConstraintPoly {
    #[fallback]
    fn compute(&mut self, _calldata: &[u8]) -> ArbResult {
        if self.preparer_address.get().is_zero() {
            return Err(format!("Preparer address not set",).into());
        }
        if self.finalizer_address.get().is_zero() {
            return Err(format!("Finalizer address not set",).into());
        }
        let cp_and_domains =
            static_call(Call::new(), self.preparer_address.get(), _calldata).unwrap();
        console!("cp_and_domains: {:?}", cp_and_domains);
        let poly_data_result = static_call(
            Call::new(),
            self.finalizer_address.get(),
            [_calldata, cp_and_domains.as_slice()].concat().as_slice(),
        )
        .unwrap();

        Ok(poly_data_result)
    }

    fn set_addresses(&mut self, preparer_address: Address, finalizer_address: Address) {
        self.preparer_address.set(preparer_address);
        self.finalizer_address.set(finalizer_address);
    }
}
