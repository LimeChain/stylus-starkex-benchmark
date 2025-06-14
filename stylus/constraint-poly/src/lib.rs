//!
//! The address `0x05` refers to a built-in EVM ModExp precompile contract.
//! https://eips.ethereum.org/EIPS/eip-198
//! - prepare an array of 48 modular exponentiations
//!   - Simple Exponentiations (via expmod) expmods[0], 2, 6, 9, 24, 40–47 (total ≈ 17)
//!   -  Derived Powers (via mulmod)
//! - Compute domains.
//! - Prepare denominators for batch inverse.
//! - Compute the inverses of the denominators into denominatorInvs using batch inverse.
//! - Compute the result of the composition polynomial. the most expensive part.
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::alloy_primitives::{address, hex, uint, Address, U256};
use stylus_sdk::call::{static_call, Call};
use stylus_sdk::console;
use stylus_sdk::stylus_core::calls::errors::Error;
use stylus_sdk::{prelude::*, ArbResult};

// debug imports

const PRIME: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256);
const TRACE_LEN_IDX: usize = 7;
const TRACE_GENERATOR_IDX: usize = 33;
const OODS_POINT_INDX: usize = 34;
const EXPECTED_INPUT_LEN: usize = 0x1d40; // 7488 bytes
#[storage]
#[entrypoint]
pub struct ConstraintPoly;

#[public]
impl ConstraintPoly {
    #[fallback]
    fn compute(&mut self, _calldata: &[u8]) -> ArbResult {
        if _calldata.len() != EXPECTED_INPUT_LEN {
            return Err(format!("Invalid calldata length: {}", _calldata.len()).into());
        }

        let calldata_words: Vec<U256> = _calldata.chunks(32).map(U256::from_be_slice).collect();
        let point = calldata_words[OODS_POINT_INDX];

        let trace_len = calldata_words[TRACE_LEN_IDX];
        let trace_generator = calldata_words[TRACE_GENERATOR_IDX];

        let expmods = match Self::make_expmods(trace_len, point, trace_generator) {
            Ok(expmods) => expmods,
            Err(e) => {
                return Err(format!("Error making expmods: {:?}", e).into());
            }
        };

        console!(
            "expmods: 0x{}",
            hex::encode(
                &expmods
                    .iter()
                    .map(|e| e.to_be_bytes::<32>())
                    .collect::<Vec<[u8; 32]>>()
                    .concat()
            )
        );

        Ok(expmods
            .iter()
            .map(|e| e.to_be_bytes::<32>())
            .collect::<Vec<[u8; 32]>>()
            .concat())
    }
}

impl ConstraintPoly {
    #[inline(always)]
    pub fn make_expmod_input(base: U256, exponent: U256) -> Vec<u8> {
        console!(
            "make_expmod_input started: base: 0x{}, exponent: 0x{}",
            hex::encode(&base.to_be_bytes::<32>()),
            hex::encode(&exponent.to_be_bytes::<32>())
        );
        let mut input = Vec::new();

        // Length fields (32 bytes each)
        input.extend_from_slice(&U256::from(32).to_be_bytes::<32>()); // base length
        input.extend_from_slice(&U256::from(32).to_be_bytes::<32>()); // exponent length
        input.extend_from_slice(&U256::from(32).to_be_bytes::<32>()); // modulus length

        // Value fields (32 bytes each)
        input.extend_from_slice(&base.to_be_bytes::<32>()); // base value
        input.extend_from_slice(&exponent.to_be_bytes::<32>()); // exponent value
        input.extend_from_slice(&PRIME.to_be_bytes::<32>()); // modulus value (PRIME)

        console!(
            "make_expmod_input finished: input: 0x{}",
            hex::encode(&input)
        );
        input
    }

    #[inline(always)]
    pub fn expmod(base: U256, exponent: U256) -> Result<U256, Error> {
        console!("expmod started");

        let result_bytes = static_call(
            Call::new(),
            address!("0000000000000000000000000000000000000005"),
            &Self::make_expmod_input(base, exponent),
        )
        .expect("modexp precompile failed");
        if result_bytes.len() != 32 {
            return Err(Error::Revert(
                "modexp precompile returned invalid length".into(),
            ));
        }
        Ok(U256::from_be_slice(&result_bytes))
    }

    /// Prepares a vector of modular exponentiations for the constraint polynomial.
    pub fn make_expmods(
        trace_length: U256,
        point: U256,
        trace_generator: U256,
    ) -> Result<Vec<U256>, Error> {
        let mut expmods = Vec::<U256>::with_capacity(48);

        // expmods[0] = point^(trace_length / 2048)
        let e0 = Self::expmod(point, trace_length / uint!(2048_U256))?;
        expmods.push(e0);

        // expmods[1] = e0^2
        let e1 = e0.mul_mod(e0, PRIME);
        expmods.push(e1);

        // expmods[2] = point^(trace_length / 128)
        let e2 = Self::expmod(point, trace_length / uint!(128_U256))?;
        expmods.push(e2);

        // expmods[3] = e2^2
        let e3 = e2.mul_mod(e2, PRIME);
        expmods.push(e3);

        // expmods[4] = e3^2
        let e4 = e3.mul_mod(e3, PRIME);
        expmods.push(e4);

        // expmods[5] = e4^2
        let e5 = e4.mul_mod(e4, PRIME);
        expmods.push(e5);

        // expmods[6] = point^(trace_length / 4)
        let e6 = Self::expmod(point, trace_length / uint!(4_U256))?;
        expmods.push(e6);

        // expmods[7] = e6^2
        let e7 = e6.mul_mod(e6, PRIME);
        expmods.push(e7);

        // expmods[8] = e7^2
        let e8 = e7.mul_mod(e7, PRIME);
        expmods.push(e8);

        // expmods[9] = trace_generator^(trace_length / 64)
        let e9 = Self::expmod(trace_generator, trace_length / uint!(64_U256))?;
        expmods.push(e9);

        // expmods[10] = e9^2 = trace_generator^(trace_length / 32)
        let e10 = e9.mul_mod(e9, PRIME);
        expmods.push(e10);

        // expmods[11] = e9 * e10 % PRIME = 3 * trace_length / 64
        let e11 = e9.mul_mod(e10, PRIME);
        expmods.push(e11);

        // expmods[12] = e9 * e11 % PRIME = trace_length / 16
        let e12 = e9.mul_mod(e11, PRIME);
        expmods.push(e12);

        // expmods[13] = e9 * e12 % PRIME = 5 * trace_length / 64
        let e13 = e9.mul_mod(e12, PRIME);
        expmods.push(e13);

        // expmods[14] = e9 * e13 % PRIME = 3 * trace_length / 32
        let e14 = e9.mul_mod(e13, PRIME);
        expmods.push(e14);

        // expmods[15] = e9 * e14 % PRIME = 7 * trace_length / 64
        let e15 = e9.mul_mod(e14, PRIME);
        expmods.push(e15);

        // expmods[16] = e9 * e15 % PRIME = trace_length / 8
        let e16 = e9.mul_mod(e15, PRIME);
        expmods.push(e16);

        // expmods[17] = e9 * e16 % PRIME = 9 * trace_length / 64
        let e17 = e9.mul_mod(e16, PRIME);
        expmods.push(e17);

        // expmods[18] = e9 * e17 % PRIME = 5 * trace_length / 32
        let e18 = e9.mul_mod(e17, PRIME);
        expmods.push(e18);

        // expmods[19] = e9 * e18 % PRIME = 11 * trace_length / 64
        let e19 = e9.mul_mod(e18, PRIME);
        expmods.push(e19);

        // expmods[20] = e9 * e19 % PRIME = 3 * trace_length / 16
        let e20 = e9.mul_mod(e19, PRIME);
        expmods.push(e20);

        // expmods[21] = e9 * e20 % PRIME = 13 * trace_length / 64
        let e21 = e9.mul_mod(e20, PRIME);
        expmods.push(e21);

        // expmods[22] = e9 * e21 % PRIME = 7 * trace_length / 32
        let e22 = e9.mul_mod(e21, PRIME);
        expmods.push(e22);

        // expmods[23] = e9 * e22 % PRIME = 15 * trace_length / 64
        let e23 = e9.mul_mod(e22, PRIME);
        expmods.push(e23);

        // expmods[24] = trace_generator^(trace_length / 2)
        let e24 = Self::expmod(trace_generator, trace_length / uint!(2_U256))?;
        expmods.push(e24);

        // expmods[25] = expmods[14] * expmods[24] = 19 * trace_length / 32
        let e25 = expmods[14].mul_mod(e24, PRIME);
        expmods.push(e25);

        // expmods[26] = expmods[10] * e25 = 5 * trace_length / 8
        let e26 = expmods[10].mul_mod(e25, PRIME);
        expmods.push(e26);

        // expmods[27] = expmods[10] * e26 = 21 * trace_length / 32
        let e27 = expmods[10].mul_mod(e26, PRIME);
        expmods.push(e27);

        // expmods[28] = expmods[10] * e27 = 11 * trace_length / 16
        let e28 = expmods[10].mul_mod(e27, PRIME);
        expmods.push(e28);

        // expmods[29] = expmods[10] * e28 = 23 * trace_length / 32
        let e29 = expmods[10].mul_mod(e28, PRIME);
        expmods.push(e29);

        // expmods[30] = expmods[10] * e29 = 3 * trace_length / 4
        let e30 = expmods[10].mul_mod(e29, PRIME);
        expmods.push(e30);

        // expmods[31] = expmods[10] * e30 = 25 * trace_length / 32
        let e31 = expmods[10].mul_mod(e30, PRIME);
        expmods.push(e31);

        // expmods[32] = expmods[10] * e31 = 13 * trace_length / 16
        let e32 = expmods[10].mul_mod(e31, PRIME);
        expmods.push(e32);

        // expmods[33] = expmods[10] * e32 = 27 * trace_length / 32
        let e33 = expmods[10].mul_mod(e32, PRIME);
        expmods.push(e33);

        // expmods[34] = expmods[10] * e33 = 7 * trace_length / 8
        let e34 = expmods[10].mul_mod(e33, PRIME);
        expmods.push(e34);

        // expmods[35] = expmods[10] * e34 = 29 * trace_length / 32
        let e35 = expmods[10].mul_mod(e34, PRIME);
        expmods.push(e35);

        // expmods[36] = expmods[10] * e35 = 15 * trace_length / 16
        let e36 = expmods[10].mul_mod(e35, PRIME);
        expmods.push(e36);

        // expmods[37] = e9 * e36 = 61 * trace_length / 64
        let e37 = e9.mul_mod(e36, PRIME);
        expmods.push(e37);

        // expmods[38] = e9 * e37 = 31 * trace_length / 32
        let e38 = e9.mul_mod(e37, PRIME);
        expmods.push(e38);

        // expmods[39] = e9 * e38 = 63 * trace_length / 64
        let e39 = e9.mul_mod(e38, PRIME);
        expmods.push(e39);

        // expmods[40] = trace_generator^(255 * trace_length / 256)
        let e40 = trace_length
            .checked_mul(uint!(255_U256))
            .and_then(|v| v.checked_div(uint!(256_U256)))
            .ok_or(Error::Revert("trace_length * 255 / 256 overflowed".into()))
            .and_then(|exp| Self::expmod(trace_generator, exp))?;
        expmods.push(e40);

        // expmods[41] = trace_generator^(trace_length - 16)
        let e41 = Self::expmod(trace_generator, trace_length - uint!(16_U256))?;
        expmods.push(e41);

        // expmods[42] = trace_generator^(trace_length - 2)
        let e42 = Self::expmod(trace_generator, trace_length - uint!(2_U256))?;
        expmods.push(e42);

        // expmods[43] = trace_generator^(trace_length - 4)
        let e43 = Self::expmod(trace_generator, trace_length - uint!(4_U256))?;
        expmods.push(e43);

        // expmods[44] = trace_generator^(trace_length - 1)
        let e44 = Self::expmod(trace_generator, trace_length - uint!(1_U256))?;
        expmods.push(e44);

        // expmods[45] = trace_generator^(trace_length - 2048)
        let e45 = Self::expmod(trace_generator, trace_length - uint!(2048_U256))?;
        expmods.push(e45);

        // expmods[46] = trace_generator^(trace_length - 128)
        let e46 = Self::expmod(trace_generator, trace_length - uint!(128_U256))?;
        expmods.push(e46);

        // expmods[47] = trace_generator^(trace_length - 64)
        let e47 = Self::expmod(trace_generator, trace_length - uint!(64_U256))?;
        expmods.push(e47);

        Ok(expmods)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::hex;
    use stylus_sdk::alloy_primitives::{uint, U256};

    #[motsu::test]
    fn test_expmod_input() {
        let expected1 = hex!("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200615233634ff4ea9d9ff89cf4a6460f382b32d679d3ef86c95d917661c7df5bf0800000000000010ffffffffffffffffffffffffffffffffffffffffffffffff0800000000000011000000000000000000000000000000000000000000000001");
        let expected2 = hex!("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002003d8d2c79e51225ca679e36b4795d34603148f22aa2da68432609f1d4586dbc300000000000000000000000000000000000000000000000000000000020000000800000000000011000000000000000000000000000000000000000000000001");
        let expected3 = hex!("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002004c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad00000000000000000000000000000000000000000000000000000000000080000800000000000011000000000000000000000000000000000000000000000001");

        let base = uint!(0x0615233634ff4ea9d9ff89cf4a6460f382b32d679d3ef86c95d917661c7df5bf_U256);
        let exponent =
            uint!(0x0800000000000010ffffffffffffffffffffffffffffffffffffffffffffffff_U256);

        let input = ConstraintPoly::make_expmod_input(base, exponent);

        // Verify total length: 6 * 32 bytes = 192 bytes
        assert_eq!(input.len(), 192);

        // Compare with expected hex string
        assert_eq!(input, expected1);
        println!("Input hex: 0x{}", hex::encode(&input));

        let base2 = uint!(0x03d8d2c79e51225ca679e36b4795d34603148f22aa2da68432609f1d4586dbc3_U256);
        let exponent2 = uint!(0x2000000_U256);

        let input2 = ConstraintPoly::make_expmod_input(base2, exponent2);

        // Verify format consistency
        assert_eq!(input2.len(), 192);
        println!("Input2 hex: 0x{}", hex::encode(&input2));

        assert_eq!(input2, expected2);

        let base3 = uint!(0x04c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad_U256);
        let exponent3 = uint!(0x8000_U256);

        let input3 = ConstraintPoly::make_expmod_input(base3, exponent3);

        println!("Input3 hex: 0x{}", hex::encode(&input3));
        assert_eq!(input3.len(), 192);
        assert_eq!(input3, expected3);
    }
}
