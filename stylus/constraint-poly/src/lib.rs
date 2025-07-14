// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::alloy_primitives::{Address, U256};
use stylus_sdk::{prelude::*, storage::StorageAddress};

mod interfaces;
use crate::interfaces::{IConstraintPolyPreparer, IConstraintPolyFinalizer};

// debug imports

#[storage]
#[entrypoint]
pub struct ConstraintPoly {
    preparer_address: StorageAddress,
    finalizer_address: StorageAddress,
}

#[public]
impl ConstraintPoly {
    
    #[inline]
    fn compute(&mut self, _calldata: Vec<U256>) -> Result<U256, Vec<u8>> {
        if self.preparer_address.get().is_zero() {
            return Err(format!("Preparer address not set",).into());
        }
        if self.finalizer_address.get().is_zero() {
            return Err(format!("Finalizer address not set",).into());
        }
        
        let preparer: IConstraintPolyPreparer = IConstraintPolyPreparer { address: self.preparer_address.get() };
        let finalizer: IConstraintPolyFinalizer = IConstraintPolyFinalizer { address: self.finalizer_address.get() };
        let cp_and_domains = preparer.compute(&mut *self, _calldata.clone())?;
        let test = [_calldata, cp_and_domains].concat();
       
        let poly_data_result = finalizer.compute(&mut *self, test)?;
        Ok(poly_data_result)
    }

    fn set_addresses(&mut self, preparer_address: Address, finalizer_address: Address) {
        self.preparer_address.set(preparer_address);
        self.finalizer_address.set(finalizer_address);
    }
}
