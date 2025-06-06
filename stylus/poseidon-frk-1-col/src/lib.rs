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
const COEFFS: [U256; 8] = [
    uint!(0x17190a2c4fe2fb2a1c4061a3aaa8d89e8a363f653a905e43ab819ff47516c67_U256),
    uint!(0x67fa64d83009acfaae5a7a0e910d322b5d4dbc825090c1239dc68cd18338ed4_U256),
    uint!(0x21052369229137423604dbda64cdab20290c4da86882c0444750eaf0687d1c8_U256),
    uint!(0x26315e8a17d10270d98790f94772ab99b185baeab1e0ec64e783de5c5b35859_U256),
    uint!(0x16ba64f5ffc9bcb3a71b49f79a1c26ce608e33f1b6ce5fdfeae1c732b5d0b5_U256),
    uint!(0x4430620ab3eb75b8b2c3ee9c8bafd3408efbe93661f670002b3f96d354c2bc0_U256),
    uint!(0x143ce163d9e857b549efa236512d839954411bc04e888aa114215f991ee8a57_U256),
    uint!(0x587584d86e310744ac2167594e87c72847cc1018d766c61b29b572ba4552a80_U256),
];
// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.

#[storage]
#[entrypoint]
pub struct PoseidonPoseidonFullRoundKey1Column;

#[public]
impl PoseidonPoseidonFullRoundKey1Column {
    pub fn compute(x: U256) -> U256 {
        let result: U256 = COEFFS[6]
            .mul_mod(x, PRIME)
            .wrapping_add(COEFFS[5])
            .mul_mod(x, PRIME)
            .wrapping_add(COEFFS[4])
            .mul_mod(x, PRIME)
            .wrapping_add(COEFFS[3])
            .mul_mod(x, PRIME)
            .wrapping_add(COEFFS[2])
            .mul_mod(x, PRIME)
            .wrapping_add(COEFFS[1])
            .mul_mod(x, PRIME)
            .wrapping_add(COEFFS[0])
            .mul_mod(x, PRIME)
            .wrapping_add(COEFFS[7]);

        result % PRIME
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
            1664257228653772301912891197477956780973260593455413394763471271235501957228_U256
        );

        assert_eq!(PoseidonPoseidonFullRoundKey1Column::compute(x), expected);
    }
}
