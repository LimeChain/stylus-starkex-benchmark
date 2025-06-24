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
    uint!(0x7d384f90e1f21f53dbafb1648ecdb97d8c020dbad501b0d79a491587484fefa_U256);
const COEFFS: [U256; 7] = [
    uint!(0x646004831088eedddafcec3518108e2033e3e613eb2b2b0ca972f75946901ba_U256),
    uint!(0x71a637fccbfdcc8da4828cb4734b6887fe9ebd78725ceb92d2756ea4e4c86fb_U256),
    uint!(0x2fa9daffc6ffa8c6dd8cf633aa7c2d2a113a885f4ba935ff7f0198a4ea056cf_U256),
    uint!(0x71273291cc9fb7c500b008872a8890e1e3917ea2b954d1f4a9af67427323126_U256),
    uint!(0x27a6021b1b06d9adf868d5ba9b068ecdee5e65fe62163095b96f7f4c2fa6c3e_U256),
    uint!(0x6217cc4bd0f62fec8a25f305b3914f3c6c2df7701aee105c60cd37ef815239a_U256),
    uint!(0x565a88ff293c0a9c48cb67be157ad800604990d390e1b173e9bdc09abf9f788_U256),
];

#[storage]
#[entrypoint]
pub struct PoseidonPoseidonFullRoundKey2Column;

#[public]
impl PoseidonPoseidonFullRoundKey2Column {
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
            1938976483485279484363264204509611131731729867572976629648616677903267220493_U256
        );

        assert_eq!(PoseidonPoseidonFullRoundKey2Column::compute(x), expected);
    }
}
