//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::alloy_primitives::{uint, U256};
use stylus_sdk::prelude::*;

const PRIME: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256);

const COEFFS_COMBINED: [U256; 32] = [
    // Block 1
    uint!(0x26f2aa4059eb10ba60302d001cdf4a5482d43e2d7d05bd2b5486cd8c52ab9be_U256),
    uint!(0x722090545903a2f0b654199a04a5db8fc128eb36cbad8255818bf1d5db2736d_U256),
    uint!(0x7510614da9b9ad318575990ca2107d7b8b4e66622a28b08499b7444a86e0d37_U256),
    uint!(0x7658d45c2170beb301fdad273c8aea07d4add3b02890567fa38c0f6b5c1689e_U256),
    uint!(0x38ea9c051a4621f17bf1d34344272953018e378f8b587aabac79157963d7a11_U256),
    uint!(0x794689eb6cd1d1acb82b7d5741d61a961b28a3f5468403a1981ddbc21eca96d_U256),
    uint!(0x4883c98a63a118749cc26ac47607af9d17fb8fd36ccb743e2b6dc13f202a42f_U256),
    // Block 2
    uint!(0x8729cd967a805126fa9fd4136a390051c690dfc413f1de62f6fc13123f9586_U256),
    uint!(0x17dae5cd6089cc03cafa39762a14985af1e7a05e9bbf55d3952c86839098c06_U256),
    uint!(0x7406ca984b25f47732349b87565103d2bbf220ebab93085c063ce5ef28e7337_U256),
    uint!(0x30e1c6b719648866af8220a2220904dd632b089e54ca459dcab5d853043fc25_U256),
    uint!(0x26993661e64b45b5787fd8b923ecfb6f681b554191429fbfd96f7010aba3115_U256),
    uint!(0x57455541d2426a546ddc818e996e5e4120a233416ce5da3422b065b60c287d3_U256),
    uint!(0x6ae8114b8a4b0e360f3108b4c4679c6e51d7870c05c3cc5504007a29c118b53_U256),
    // Block 3
    uint!(0x5acb7c9ee9cc689cf9ed6c611a1bd730f43c4ea34b94e07ed804fb6d2bb8d4f_U256),
    uint!(0x29a15985dd04254ad523298f35de868c8f4538f2d800d6005634b3a32bb00f2_U256),
    uint!(0x292b8bad037db0033c816ef6752c1bb9d551215a498452832f721cd95519372_U256),
    uint!(0x31a3edaa5ab567b05861b16a6e0da76ea8e159108d2fe83eb73ad7b8f86ef7a_U256),
    uint!(0x2d35fd2bf29729904a91cc5ebd7d79362c34828e0c37e09aa4907de26a45fb3_U256),
    uint!(0x3b1f311a53410f51c90fd8a9189465059ef46149b8fb7930963ead8eabaf53a_U256),
    uint!(0x230279d6296ebacdcc9cfa5bf60e5a1d4ebe3ff0ac8f8bf8318c988f5c99bf8_U256),
    // Block 4
    uint!(0x2369e96b64fea009a1f66290a5dfe08010918b4ce3bfc9066739a4dbe133a0d_U256),
    uint!(0x13816f7acb88c6bf0356430faf0c4fed6972a9498b29919af38d9d5f5ae440a_U256),
    uint!(0x787d78882592b85f1de17e47bad43712e69d0899fc94beff77d62d2c4a1375c_U256),
    uint!(0x1619514ce1cabc2996036ec8d3e3476a8a2d9e83be3e8aa7a020ad11b548622_U256),
    uint!(0x176e131cb6830fe0570f692f5cce9f3f37d3444e647a318f35d1138bb580133_U256),
    uint!(0x4a300aa8b63feafeec79c07fe87991c0e85737ae1fbe3aa7f60d285bcf89cf3_U256),
    uint!(0x7845d76e25e1fe6f884a162b379bad9bd7e421befbd911bfd1810a6973ca552_U256),
    // Block 5 (only 4 coefficients!)
    uint!(0x32e59ba3c11289dfbca64ae5646d50270c6f78f070e0ed6f1b24f45ce6832a6_U256),
    uint!(0x3779baab792c4c306342b4e6640c4a1c315ff9d08e0fcc97576ef889dd657b6_U256),
    uint!(0x6712fc14a35a8b515bfc975d8cf356c749b04d5d7abc78ba6b2aa0924d6146f_U256),
    uint!(0x4b7fdbd1ae93d05ecb29e4702e1715f462cce519dba31b9f4b87107ada27016_U256),
];
#[storage]
#[entrypoint]
pub struct PoseidonPoseidonPartialRoundKey1Column;

#[public]
impl PoseidonPoseidonPartialRoundKey1Column {
    pub fn compute(x: U256) -> U256 {
        COEFFS_COMBINED.iter().fold(U256::ZERO, |acc, &coeff| {
            acc.mul_mod(x, PRIME).wrapping_add(coeff)
        }) % PRIME
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
            2486570557154671379335084513491649861794821253711847039152551529444239535533_U256
        );

        assert_eq!(PoseidonPoseidonPartialRoundKey1Column::compute(x), expected);
    }
}
