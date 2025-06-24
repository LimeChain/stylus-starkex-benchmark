//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
mod coeffs;

use alloc::vec::Vec;
use coeffs::{COEFFS, COEFF_LAST};
use stylus_sdk::alloy_primitives::{uint, U256};
use stylus_sdk::prelude::*;

const PRIME: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256);
#[storage]
#[entrypoint]
pub struct PedersenHashPointsYColumn;

#[public]
impl PedersenHashPointsYColumn {
    pub fn compute(x: U256) -> U256 {
        COEFFS
            .iter()
            .fold(U256::ZERO, |acc, &coeff| {
                acc.mul_mod(x, PRIME).wrapping_add(coeff)
            })
            .mul_mod(x, PRIME)
            .wrapping_add(COEFF_LAST)
            % PRIME
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::alloy_primitives::{uint, U256};
    #[motsu::test]
    fn test_compute() {
        let x: U256 = uint!(
            2502371038239847331946845555940821891939660827069539886818086403686260021246_U256
        );

        let expected: U256 = uint!(
            1444533035788560090889078696321009507857064390212204404518903797387225515076_U256
        );

        assert_eq!(PedersenHashPointsYColumn::compute(x), expected);
    }
}
