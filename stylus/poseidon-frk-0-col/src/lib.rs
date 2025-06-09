//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::alloy_primitives::{uint, U256};
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::prelude::*;
const PRIME: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256);
const COEFF_LAST: U256 =
    uint!(0x47da67f078d657e777a79423be81a5d41f445f9455b207ec9768858cfd134f1_U256);
const COEFFS: [U256; 7] = [
    uint!(0x2574ea7cc37bd716e0ec143a2420103589ba7b2af9d6b07569af3b108450a90_U256),
    uint!(0x712a2cab5d2a48c76a95de8f29a898d655cc216172a400ca054d6eb9950d698_U256),
    uint!(0x7865d89fa1e9dce49da0ac14d7437366bd450fb823a4fd3d2d8b1726f924c8f_U256),
    uint!(0x1b8c9c9cfe3c81279569f1130da6064cbf12c4b828d7e0cf60735514cf96c22_U256),
    uint!(0x11eaccb2939fb9e21a2a44d6f1e0608aac4248f817bc9458cce8a56077a22b1_U256),
    uint!(0x5f3e9a55edfd3f6abac770ff5606fca5aaf7074bedae94ade74395453235e8e_U256),
    uint!(0x7ed6ec4a18e23340489e4e36db8f4fcebf6b6ebd56185c29397344c5deea4c8_U256),
];

#[storage]
#[entrypoint]
pub struct PoseidonPoseidonFullRoundKey0Column;

#[public]
impl PoseidonPoseidonFullRoundKey0Column {
    pub fn compute(x: U256) -> U256 {
        let result = COEFFS.iter().rev().fold(U256::ZERO, |acc, &coeff| {
            acc.mul_mod(x, PRIME).wrapping_add(coeff)
        });

        result.mul_mod(x, PRIME).wrapping_add(COEFF_LAST) % PRIME
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::alloy_primitives::{uint, U256};
    #[motsu::test]
    fn test_compute() {
        let x: U256 =
            uint!(513761785516736576210258345954495650460389361631034617172115002511570125974_U256);

        let expected: U256 = uint!(
            1747952454919021766681010400995206390562374609324430906386085649753967957996_U256
        );

        assert_eq!(PoseidonPoseidonFullRoundKey0Column::compute(x), expected);
    }
}
