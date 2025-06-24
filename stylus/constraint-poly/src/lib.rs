//!
//! The address `0x05` refers to a built-in EVM ModExp precompile contract.
//! https://eips.ethereum.org/EIPS/eip-198
//! - 48 modular exponentiations - 0x23c0 - 0x29c0
//!   - Simple Exponentiations (via expmod) expmods[0], 2, 6, 9, 24, 40–47 (total ≈ 17)
//!   -  Derived Powers (via mulmod)
//! - Compute domains                         | 0x29c0[334] - 0x2d40[362]
//! - denominators inversed                   | 0x2d40[362] - 0x2f80[380]
//! - Prepare denominators for batch inverse. | 0x2f80[380] - 0x31c0[398]
//! - Compute the inverses of the denominators into denominatorInvs using batch inverse. - 0x31a0[397] - 0x3280[408] - 0x32a0[410]
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

        // console!(
        //     "expmods: 0x{}",
        //     hex::encode(
        //         &expmods
        //             .iter()
        //             .map(|e| e.to_be_bytes::<32>())
        //             .collect::<Vec<[u8; 32]>>()
        //             .concat()
        //     )
        // );

        let domains = match ConstraintPoly::compute_domains(&expmods, point) {
            Ok(domains) => domains,
            Err(e) => {
                return Err(format!("Error computing domains: {:?}", e).into());
            }
        };

        // console!(
        //     "domains: 0x{}",
        //     hex::encode(&domains.iter().map(|e| e.to_be_bytes::<32>()).collect::<Vec<[u8; 32]>>().concat())
        // );
        let mut denominators = vec![
            domains[0],
            domains[3],
            domains[4],
            domains[20],
            domains[21],
            domains[1],
            domains[22],
            domains[2],
            domains[23],
            domains[24],
            domains[15],
            domains[16],
            domains[17],
            domains[19],
            domains[8],
            domains[5],
            domains[10],
            domains[6],
        ];

        let den_inv = match ConstraintPoly::batch_inverse(&mut denominators) {
            Ok(den_inv) => den_inv,
            Err(e) => {
                return Err(format!("Error computing batch inverse: {:?}", e).into());
            }
        };

        Ok(domains
            .iter()
            .map(|e| e.to_be_bytes::<32>())
            .collect::<Vec<[u8; 32]>>()
            .concat())
    }
}

impl ConstraintPoly {
    /// Computes the batch modular inverses of a list of denominators.
    pub fn batch_inverse(denominators: &mut [U256]) -> Result<Vec<U256>, Error> {
        let mut partial_products = Vec::with_capacity(denominators.len());
        let mut prod = U256::from(1);

        // Build partial products
        for d in denominators.iter() {
            partial_products.push(prod);
            prod = prod.mul_mod(*d, PRIME);
        }

        // Compute inverse of the total product
        let mut prod_inv = prod.pow_mod(PRIME.wrapping_sub(U256::from(2)), PRIME);
        if prod_inv.is_zero() {
            return Err(Error::Revert("Batch inverse product is zero.".into()));
        }

        // Compute inverses
        let mut inverses = vec![U256::ZERO; denominators.len()];
        for i in (0..denominators.len()).rev() {
            inverses[i] = partial_products[i].mul_mod(prod_inv, PRIME);
            prod_inv = prod_inv.mul_mod(denominators[i], PRIME);
        }

        Ok(inverses)
    }

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
        #[cfg(not(test))]
        {
            console!("expmod calling precompile");
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
            return Ok(U256::from_be_slice(&result_bytes));
        }

        #[cfg(test)]
        {
            console!("expmod calling pow_mod");
            return Ok(base.pow_mod(exponent, PRIME));
        }
    }

    fn compute_domains(expmods: &[U256], point: U256) -> Result<Vec<U256>, Error> {
        let mut domains = Vec::<U256>::with_capacity(28);
        // Helper: PRIME - val
        // let sub_prime = |val: U256| PRIME.wrapping_sub(val);
        let prime_minus_1 = PRIME.wrapping_sub(U256::from(1));

        domains.push(expmods[8].add_mod(prime_minus_1, PRIME)); // domains[0] = point^trace_length - 1.
        domains.push(expmods[7].add_mod(prime_minus_1, PRIME)); // domains[1] = point^(trace_length / 2) - 1.
        domains.push(expmods[6].add_mod(prime_minus_1, PRIME)); // domains[2] = point^(trace_length / 4) - 1.

        // domain[3] = point^(trace_length / 16) - trace_generator^(15 * trace_length / 16)
        domains.push(expmods[5].add_mod(PRIME.wrapping_sub(expmods[36]), PRIME));

        // domain[4] = point^(trace_length / 16) - 1
        domains.push(expmods[5].add_mod(prime_minus_1, PRIME));

        // domain[5] = point^(trace_length / 32) - 1
        domains.push(expmods[4].add_mod(prime_minus_1, PRIME));

        // domain[6] = point^(trace_length / 64) - 1
        domains.push(expmods[3].add_mod(prime_minus_1, PRIME));

        // domain[7] = point^(trace_length / 64) - trace_generator^(3 * trace_length / 4)
        domains.push(expmods[3].add_mod(PRIME.wrapping_sub(expmods[30]), PRIME));

        // domain[8] = point^(trace_length / 128) - 1
        domains.push(expmods[2].add_mod(prime_minus_1, PRIME));

        // domain[9] = point^(trace_length / 128) - trace_generator^(3 * trace_length / 4)
        domains.push(expmods[2].add_mod(PRIME.wrapping_sub(expmods[30]), PRIME));

        // domains[10] = (point^(trace_length / 128) - trace_generator^(trace_length / 64)) * (point^(trace_length / 128) - trace_generator^(trace_length / 32)) * (point^(trace_length / 128) - trace_generator^(3 * trace_length / 64)) * (point^(trace_length / 128) - trace_generator^(trace_length / 16)) * (point^(trace_length / 128) - trace_generator^(5 * trace_length / 64)) * (point^(trace_length / 128) - trace_generator^(3 * trace_length / 32)) * (point^(trace_length / 128) - trace_generator^(7 * trace_length / 64)) * (point^(trace_length / 128) - trace_generator^(trace_length / 8)) * (point^(trace_length / 128) - trace_generator^(9 * trace_length / 64)) * (point^(trace_length / 128) - trace_generator^(5 * trace_length / 32)) * (point^(trace_length / 128) - trace_generator^(11 * trace_length / 64)) * (point^(trace_length / 128) - trace_generator^(3 * trace_length / 16)) * (point^(trace_length / 128) - trace_generator^(13 * trace_length / 64)) * (point^(trace_length / 128) - trace_generator^(7 * trace_length / 32)) * (point^(trace_length / 128) - trace_generator^(15 * trace_length / 64)) * domain8.
        {
            let mut d10 = U256::ONE;
            for i in 9..24 {
                d10 = d10.mul_mod(
                    expmods[2].add_mod(PRIME.wrapping_sub(expmods[i]), PRIME),
                    PRIME,
                );
            }

            // Multiply by domains[8]
            d10 = d10.mul_mod(domains[8], PRIME);
            domains.push(d10);
        }
        // domains[11] = point^(trace_length / 128) - trace_generator^(31 * trace_length / 32).
        domains.push(expmods[2].add_mod(PRIME.wrapping_sub(expmods[38]), PRIME));

        // Numerator for constraints: 'poseidon/poseidon/partial_rounds_state1_squaring'.
        // domains[12] = (point^(trace_length / 128) - trace_generator^(11 * trace_length / 16)) * (point^(trace_length / 128) - trace_generator^(23 * trace_length / 32)) * (point^(trace_length / 128) - trace_generator^(25 * trace_length / 32)) * (point^(trace_length / 128) - trace_generator^(13 * trace_length / 16)) * (point^(trace_length / 128) - trace_generator^(27 * trace_length / 32)) * (point^(trace_length / 128) - trace_generator^(7 * trace_length / 8)) * (point^(trace_length / 128) - trace_generator^(29 * trace_length / 32)) * (point^(trace_length / 128) - trace_generator^(15 * trace_length / 16)) * domain9 * domain11.
        // 0x2740 - expmods[28]
        // 0x2760 - expmods[29]
        // 0x27a0 - expmods[31]
        // 0x27c0 - expmods[32]
        // 0x27e0 - expmods[33]
        // 0x2800 - expmods[34]
        // 0x2820 - expmods[35]
        // 0x2840 - expmods[36]
        let p128 = expmods[2]; // point^(trace_length / 128)
        {
            let sub_indices = [28, 29, 31, 32, 33, 34, 35, 36];

            let mut d12 = U256::ONE;
            for &i in sub_indices.iter() {
                d12 = d12.mul_mod(p128.add_mod(PRIME.wrapping_sub(expmods[i]), PRIME), PRIME);
            }

            // Multiply by domains[9] and domains[11]
            d12 = d12.mul_mod(domains[9], PRIME).mul_mod(domains[11], PRIME);
            domains.push(d12);
        }

        // domains[13] = (expmods[2] - expmods[37]) * (expmods[2] - expmods[39]) * domains[11]
        domains.push(
            p128.add_mod(PRIME.wrapping_sub(expmods[37]), PRIME)
                .mul_mod(p128.add_mod(PRIME.wrapping_sub(expmods[39]), PRIME), PRIME)
                .mul_mod(domains[11], PRIME),
        );

        // domains[14] = (expmods[2] - expmods[25]) * (expmods[2] - expmods[26]) * (expmods[2] - expmods[27]) * domains[12]
        domains.push(
            p128.add_mod(PRIME.wrapping_sub(expmods[25]), PRIME)
                .mul_mod(p128.add_mod(PRIME.wrapping_sub(expmods[26]), PRIME), PRIME)
                .mul_mod(p128.add_mod(PRIME.wrapping_sub(expmods[27]), PRIME), PRIME)
                .mul_mod(domains[12], PRIME),
        );

        // domains[15] = point^(trace_length / 1024) - 1.
        domains.push(expmods[1].add_mod(prime_minus_1, PRIME));

        // domains[16] = point^(trace_length / 1024) - trace_generator^(255 * trace_length / 256).
        domains.push(expmods[1].add_mod(PRIME.wrapping_sub(expmods[40]), PRIME));

        // domains[17] = point^(trace_length / 1024) - trace_generator^(trace_length - 16).
        domains.push(expmods[1].add_mod(PRIME.wrapping_sub(expmods[39]), PRIME));

        // domains[18]
        domains.push(expmods[0].add_mod(PRIME.wrapping_sub(expmods[24]), PRIME));

        // domains[19]
        domains.push(expmods[0].add_mod(PRIME.wrapping_sub(U256::ONE), PRIME));

        // domains[20]
        domains.push(point.add_mod(PRIME.wrapping_sub(expmods[41]), PRIME));

        // domains[21]
        domains.push(point.add_mod(PRIME.wrapping_sub(U256::ONE), PRIME));

        // domains[22]
        domains.push(point.add_mod(PRIME.wrapping_sub(expmods[42]), PRIME));

        // domains[23]
        domains.push(point.add_mod(PRIME.wrapping_sub(expmods[43]), PRIME));

        // domains[24]
        domains.push(point.add_mod(PRIME.wrapping_sub(expmods[44]), PRIME));

        // domains[25]
        domains.push(point.add_mod(PRIME.wrapping_sub(expmods[45]), PRIME));

        // domains[26]
        domains.push(point.add_mod(PRIME.wrapping_sub(expmods[46]), PRIME));

        // domains[27]
        domains.push(point.add_mod(PRIME.wrapping_sub(expmods[47]), PRIME));

        Ok(domains)
    }
    /// Prepares a vector of modular exponentiations for the constraint polynomial.
    pub fn make_expmods(
        trace_length: U256,
        point: U256,
        trace_generator: U256,
    ) -> Result<Vec<U256>, Error> {
        let mut expmods = Vec::<U256>::with_capacity(48);

        // expmods[0] = point^(trace_length / 2048)
        // 0x23c0
        let e0 = Self::expmod(point, trace_length / uint!(2048_U256))?;
        expmods.push(e0);

        // expmods[1] = point^(trace_length / 1024)
        // 0x23e0
        expmods.push(e0.mul_mod(e0, PRIME));

        // expmods[2] = point^(trace_length / 128)
        // 0x2400
        let e2 = Self::expmod(point, trace_length / uint!(128_U256))?;
        expmods.push(e2);

        // expmods[3] = point^(trace_length / 64).
        // 0x2420
        expmods.push(e2.mul_mod(e2, PRIME));

        // expmods[4] = point^(trace_length / 8)
        // 0x2440
        expmods.push(expmods[3].mul_mod(expmods[3], PRIME));

        // expmods[5] = point^(trace_length / 16)
        // 0x2460
        expmods.push(expmods[4].mul_mod(expmods[4], PRIME));

        // expmods[6] = point^(trace_length / 4)
        // 0x2480
        let e6 = Self::expmod(point, trace_length / uint!(4_U256))?;
        expmods.push(e6);

        // expmods[7] = point^(trace_length / 2).
        // 0x24a0
        expmods.push(e6.mul_mod(e6, PRIME));

        // expmods[8] = point^trace_length.
        // 0x24c0
        expmods.push(expmods[7].mul_mod(expmods[7], PRIME));

        // expmods[9] = trace_generator^(trace_length / 64)
        let e9 = Self::expmod(trace_generator, trace_length / uint!(64_U256))?;
        expmods.push(e9);

        // expmods[10] = e9^2 = trace_generator^(trace_length / 32)
        expmods.push(e9.mul_mod(e9, PRIME));

        // expmods[11] = e9 * e10 % PRIME = 3 * trace_length / 64
        expmods.push(e9.mul_mod(expmods[10], PRIME));

        // expmods[12] = e9 * e11 % PRIME = trace_length / 16
        expmods.push(e9.mul_mod(expmods[11], PRIME));

        // expmods[13] = e9 * e12 % PRIME = 5 * trace_length / 64
        expmods.push(e9.mul_mod(expmods[12], PRIME));

        // expmods[14] = e9 * e13 % PRIME = 3 * trace_length / 32
        expmods.push(e9.mul_mod(expmods[13], PRIME));

        // expmods[15] = e9 * e14 % PRIME = 7 * trace_length / 64
        expmods.push(e9.mul_mod(expmods[14], PRIME));

        // expmods[16] = e9 * e15 % PRIME = trace_length / 8
        expmods.push(e9.mul_mod(expmods[15], PRIME));

        // expmods[17] = e9 * e16 % PRIME = 9 * trace_length / 64
        expmods.push(e9.mul_mod(expmods[16], PRIME));

        // expmods[18] = e9 * e17 % PRIME = 5 * trace_length / 32
        expmods.push(e9.mul_mod(expmods[17], PRIME));

        // expmods[19] = e9 * e18 % PRIME = 11 * trace_length / 64
        expmods.push(e9.mul_mod(expmods[18], PRIME));

        // expmods[20] = e9 * e19 % PRIME = 3 * trace_length / 16
        expmods.push(e9.mul_mod(expmods[19], PRIME));

        // expmods[21] = trace_generator^(13 * trace_length / 64).
        // 0x2660
        expmods.push(e9.mul_mod(expmods[20], PRIME));

        // expmods[22] = trace_generator^(7 * trace_length / 32).
        // 0x2680
        expmods.push(e9.mul_mod(expmods[21], PRIME));

        // expmods[23] = trace_generator^(15 * trace_length / 64).
        // 0x26a0
        expmods.push(e9.mul_mod(expmods[22], PRIME));

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
    use std::os::unix::raw::uid_t;

    use super::*;
    use alloy_primitives::hex;
    use stylus_sdk::alloy_primitives::{uint, U256};

    // #[motsu::test]
    // fn test_expmod_input() {
    //     let expected1 = hex!("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200615233634ff4ea9d9ff89cf4a6460f382b32d679d3ef86c95d917661c7df5bf0800000000000010ffffffffffffffffffffffffffffffffffffffffffffffff0800000000000011000000000000000000000000000000000000000000000001");
    //     let expected2 = hex!("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002003d8d2c79e51225ca679e36b4795d34603148f22aa2da68432609f1d4586dbc300000000000000000000000000000000000000000000000000000000020000000800000000000011000000000000000000000000000000000000000000000001");
    //     let expected3 = hex!("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002004c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad00000000000000000000000000000000000000000000000000000000000080000800000000000011000000000000000000000000000000000000000000000001");

    //     let base = uint!(0x0615233634ff4ea9d9ff89cf4a6460f382b32d679d3ef86c95d917661c7df5bf_U256);
    //     let exponent =
    //         uint!(0x0800000000000010ffffffffffffffffffffffffffffffffffffffffffffffff_U256);

    //     let input = ConstraintPoly::make_expmod_input(base, exponent);

    //     // Verify total length: 6 * 32 bytes = 192 bytes
    //     assert_eq!(input.len(), 192);

    //     // Compare with expected hex string
    //     assert_eq!(input, expected1);
    //     println!("Input hex: 0x{}", hex::encode(&input));

    //     let base2 = uint!(0x03d8d2c79e51225ca679e36b4795d34603148f22aa2da68432609f1d4586dbc3_U256);
    //     let exponent2 = uint!(0x2000000_U256);

    //     let input2 = ConstraintPoly::make_expmod_input(base2, exponent2);

    //     // Verify format consistency
    //     assert_eq!(input2.len(), 192);
    //     println!("Input2 hex: 0x{}", hex::encode(&input2));

    //     assert_eq!(input2, expected2);

    //     let base3 = uint!(0x04c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad_U256);
    //     let exponent3 = uint!(0x8000_U256);

    //     let input3 = ConstraintPoly::make_expmod_input(base3, exponent3);

    //     println!("Input3 hex: 0x{}", hex::encode(&input3));
    //     assert_eq!(input3.len(), 192);
    //     assert_eq!(input3, expected3);
    // }

    const DOMAINS: [U256; 28] = uint!([
        0x0717cef815ffd73e01300e4c4b518bebb8692c5a7381e2b84f05cc91d07ffe78_U256, // 334
        0x0474b97bd62ecfe1178d9c28cc08df94663431591d0815584d6d90f5ef2f37f5_U256,
        0x05ac18db25de8806876330b0332367166a16103dbad21b4b62ea11aef47286fa_U256,
        0x0757a5d6870e08eef8f1b3893da69038a2245732cf34a7bca6b7989fee3172bd_U256,
        0x051b92e2f678cfbb5d6e4f2d2218ac575364691d0c4bf40f7e4fb4fb6f771d30_U256,
        0x0096361bba64eeeb0b13222e43556627763c0b9591bad4960235606582a46c27_U256,
        0x05f17e756b27c5cd0c6094d4c591c88e4232b68583bcf16c9ea494c3213b5609_U256,
        0x041680ae94ca5f175f91b53b08bbc156075944dc30401246c905faea154e8dec_U256,
        0x0107db1e61a9fe018a87ab117832ddefaad2711ea1a6ed0837b677b46e881f1a_U256,
        0x072cdd578b4c975cddb8cb77bb5cd6b76ff8ff754e2a0de26217dddb629b56fe_U256,
        0x0079ec07136dde7180fa859206bdeaac407423f3b632b6609953cd27b81bf0b9_U256,
        0x055611b43ce297758c15000c57118552be9dd462d2c5335799cdb1e5007c2fdb_U256,
        0x045916261d24373de820ee869328132ebd593dc80c940f5053e0de4e90be8518_U256,
        0x029438262cea9b577f030ad76ce2d1028fd26af065b2dec3aead9f8ea9586cf5_U256,
        0x03c8a1c5d6d71d0aae995efacf1cc5bce21241aa188a4c83973c8e27157bd588_U256,
        0x0571a193b64de06cbd7d24931ccdd6a42a2c10fd6f7c2a61d040e3ab2b4f64ff_U256,
        0x02b03fd6e74f516e3a257c93bf38ace0bc677119e2d727cdb56d68b75aa8b00b_U256,
        0x043d766a6f9c99958a32a0e79daf0f459dce8bcbb9dbefad17c88f0d06e7fe28_U256,
        0x029c4e1a5097b18b451a2bb5af911a8a1cd84c2c26a83f06fce9630376092de6_U256,
        0x029c4e1a5097b18b451a2bb5af911a8a1cd84c2c26a83f06fce9630376092de4_U256,
        0x0321ae132e4d8ba0f073018c63c29ec91ecda4ad527630cd09b6d2eeeb18af93_U256,
        0x04c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ac_U256,
        0x051a0efa92268ccb83880b596cf95c9146fe569e88aee63eb29477acc16ae79b_U256,
        0x057a1b052ad68b4a8b5aa303d9c25d877fc5c19ed8367affd20a1b8ea82df74b_U256,
        0x0118e4c6681fcb86fee454d9b2e3df0354715c93916930a53ef136d86519b99c_U256,
        0x0131befd94d6c6c7c9072d0a08bd3a5ea7f68bc00f9b94371d0c60af27054d41_U256,
        0x05a74aa929893f985e810344eff915cfa7fc4348176fc45f2691fa75b2b5b3a9_U256,
        0x016068c68d3917811175ab74152205a07bec1ffb49f1ed0f9d153e3d57631f07_U256, // 361
    ]);

    const DEN_INV: [U256; 18] = uint!([
        0x043598cd676e8856f5f15113a27d136a3a649a1767742de5553e03cbee718aed_U256, //362
        0x0249d43eb4434663bf327c08c478644a29a2a07e772f3423855bf6f9cbb26c8c_U256, //363
        0x031e9dfe76b20201e71d1a3e0297ac6f58648ff6d73fb212609fb68cb89bd18d_U256, //364
        0x04ec71496111bc811f5af76a0fb9ea4416be94713f043ae98d3ddae0c14dd1ba_U256, //365
        0x068282a3bb956711f20a4f6490d42a27f448d18aa69e399f996e560307a831ae_U256, //366
        0x05e13b50e113d32e22aa43a012747c60d847940c1867a8d9c51c33b5d54aa8e7_U256, //367
        0x05c2dd0cc1ef02be001fd0185f599cdce133f4d84b5663f14a4962c9813a6aea_U256, //368
        0x06119eb76ef975054d4529c1e089e874d9c203d613e52e79e5d9d3793cf1d116_U256, //369
        0x040ffc62dcfb4652a60d83ea869ac432ebd50c1e0fc73809561bbdba4944cb84_U256, //370
        0x031d2fe890ef530165fb1e2960dc85d23c433e1b78c9785942432659d69eda43_U256, //371
        0x03f8f50bb49071a3009fbc2915cec18cfc5673d5aa52e6e153133a3b0fcf69e2_U256, //372
        0x0540960a9086c9f73756e7983ccf7ce7de3c79b12c0eb521eaf485212002547e_U256, //373
        0x02d8d44c2e1e5461070cca04f780bf4fe6a0e07a8b692daef8de6d8d23ec0498_U256, //374
        0x07124062f4d5ab8faef293d7a84bce995980b290b8bdb83f31916a269eed3f78_U256, //375
        0x03343aa7425f8dba86928a2e5f0ba3fcbbb8ec311749149065cc8dc96274d705_U256, //376
        0x0482bfbe94ea90a20561a0205ae4611100273774c0acd9d1cb2f28add17f2363_U256, //377
        0x0126bfa984dee7629f8dbffc80cb57f942a137d769b269c62a47a6bc02668e27_U256, //378
        0x06abd9c124608bd5c31a92d5863cfbdee1dea2df867290ef3efd90d30f3f4885_U256, //379
    ]);

    const EXPODS: [U256; 48] = uint!([
        0x029c4e1a5097b18b451a2bb5af911a8a1cd84c2c26a83f06fce9630376092de5_U256, //286
        0x0571a193b64de06cbd7d24931ccdd6a42a2c10fd6f7c2a61d040e3ab2b4f6500_U256, //287
        0x0107db1e61a9fe018a87ab117832ddefaad2711ea1a6ed0837b677b46e881f1b_U256, //288
        0x05f17e756b27c5cd0c6094d4c591c88e4232b68583bcf16c9ea494c3213b560a_U256, //289
        0x0096361bba64eeeb0b13222e43556627763c0b9591bad4960235606582a46c28_U256, //290
        0x051b92e2f678cfbb5d6e4f2d2218ac575364691d0c4bf40f7e4fb4fb6f771d31_U256, //291
        0x05ac18db25de8806876330b0332367166a16103dbad21b4b62ea11aef47286fb_U256, //292
        0x0474b97bd62ecfe1178d9c28cc08df94663431591d0815584d6d90f5ef2f37f6_U256, //293
        0x0717cef815ffd73e01300e4c4b518bebb8692c5a7381e2b84f05cc91d07ffe79_U256, //294
        0x0128f0fee82b2bb55e869a0710826800d09bd064f9e225ecd6871506b2703765_U256, //295
        0x0789ad459ecd5c85fcdca219ce6246af26da375d1a8e79812225638f9b48a8ab_U256, //296
        0x05afc640ff0f57b9b267655f3da7c64c4a910d81023e4d8ab2cf6b0e179c904a_U256, //297
        0x05ec467b88826aba4537602d514425f3b0bdf467bbf302458337c45f6021e539_U256, //298
        0x03a3bf0c4876db92b342a839378d12b97ec35d5bb74beecd3c616e4e34cf48fe_U256, //299
        0x02c226e9010da226650d4e831a8c21933d6d16ce5fad48839e202fdbb6c986d7_U256, //300
        0x04a44b0df399815cc4b2a4e3c2f8755f750774e2e9ec857782aa7feba1d98b1e_U256, //301
        0x063365fe0de874d9c90adb1e2f9c676e98c62155e4412e873ada5e1dee6feebb_U256, //302
        0x060150b421a2127371ca3d710511a6a9299ff03be41f621dd1bad555c776df4d_U256, //303
        0x0211c88e2dd40bd3cdd1c69103a26c06339749cf6943864821fdc6d10263070c_U256, //304
        0x011e64c83c6d5798845170fdada655ccc928c1c64e3bb3de3b6d627083055f32_U256, //305
        0x000b54759e8c46e1258dc80f091e6f3be387888015452ce5f0ca09ce9e571f52_U256, //306
        0x03f0af1b9a3b60e14b2af2b95e1bb124c8e30a5f0158e52f348a84263bafbe3d_U256, //307
        0x035b01f2f03b33a0e6416301ed24191cb5b5db5ad8831dda32f035d1bcc1cc71_U256, //308
        0x05863bb78599ee2d50aaaffdb3832b88493a0eca3a8773991c69d93c79f23534_U256, //309
        0x0800000000000011000000000000000000000000000000000000000000000000_U256, //310
        0x053dd916fef25dea9af2b17ce573de6cc292e931a052b77c61dfd0244936792a_U256, //311
        0x01cc9a01f2178b3736f524e1d06398916739deaa1bbed178c525a1e211901146_U256, //312
        0x05ee3771d22bf43d322e396efc5d93f9cc68b63096bc79b7de02392efd9cf8f5_U256, //313
        0x07f4ab8a6173b92fda7237f0f6e190c41c78777feabad31a0f35f63161a8e0af_U256, //314
        0x04a4fe0d0fc4cc7019be9cfe12dbe6e34a4a24a5277ce225cd0fca2e433e3390_U256, //315
        0x01dafdc6d65d66b5accedf99bcd607383ad971a9537cdf25d59e99d90becc81e_U256, //316
        0x04ae7c0d2777f18575abe8eb7ad8cac1b943249f8a2edc14d3223788632351ac_U256, //317
        0x0231c05e93ca34c35ac88ac98a35cd89152dbfa622215d35b83c9a781a5ac730_U256, //318
        0x03eefb52c4063b8e96cbb9ae685b17a16fd4f846707a5de194e93c1e6b25118c_U256, //319
        0x0446ed3ce295dda2b5ea677394813e6eab8bfbc55397aacac8e6df6f4bc9ca34_U256, //320
        0x0179fed001a9673b060f02a4e8373a030593719019b28dea4f059b03071988ec_U256, //321
        0x05c3ed0c6f6ac6dd647c9ba3e4721c1eb14011ea3d174c52d7981c5b8145aa75_U256, //322
        0x068b76e685a1afbb3fa1f335583936958214264afec146eb863bf481aa1e8a0c_U256, //323
        0x03b1c96a24c7669cfe72ab052121589cec349cbbcee1b9b09de8c5cf6e0bef41_U256, //324
        0x01342b2946b146d7334a83ab7f1ec75e8c5d8531b5a03ab4b878549e246766d8_U256, //325
        0x02c161bccefe8efe8357a7ff5d9529c36dc49fe38ca502941ad37af3d0a6b4f5_U256, //326
        0x019e8e437bd95ece45cf52e13abd5bfd4266bc82a085c456785975cc2e35511a_U256, //327
        0x07a62d5c18005db4b2ba491431869e351a360a916a4d0ee4cf7bd10e57e31913_U256, //328
        0x074621517f505f35aae7b169c4bd9d3ee16e9f911ac57a23b0062d2c71200963_U256, //329
        0x03a7579042071ee8375dff93eb9c1bc30cc3049c6192c47e431f11e2b4344711_U256, //330
        0x038e7d59155023a76d3b276395c2c067b93dd56fe36060ec6503e80bf248b36c_U256, //331
        0x0718f1ad809daae7d7c15128ae86e4f6b9381de7db8c30c45b7e4e4566984d05_U256, //332
        0x035fd3901cedd2ee24cca8f9895df525e5484134a90a0813e4fb0a7dc1eae1a6_U256, //333
    ]);
    #[motsu::test]
    fn test_expmod() {
        let trace_length =
            uint!(0x0000000000000000000000000000000000000000000000000000000004000000_U256);
        let point = uint!(0x04c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad_U256);
        let trace_generator =
            uint!(0x03d8d2c79e51225ca679e36b4795d34603148f22aa2da68432609f1d4586dbc3_U256);

        let result = ConstraintPoly::make_expmods(trace_length, point, trace_generator).unwrap();

        assert_eq!(result.len(), EXPODS.len());

        for (i, expod) in EXPODS.iter().enumerate() {
            assert_eq!(result[i], *expod, "expod[{}] is wrong", i);
        }
    }

    #[motsu::test]
    fn test_compute_domains() {
        let point = uint!(0x04c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad_U256);
        let domains = ConstraintPoly::compute_domains(&EXPODS, point).unwrap();
        println!("domains: {:?}", domains.len());
        println!(
            "domains[0]: 0x{}",
            hex::encode(&domains[0].to_be_bytes::<32>())
        );
        println!(
            "domains[1]: 0x{}",
            hex::encode(&domains[1].to_be_bytes::<32>())
        );
        assert_eq!(domains[0], DOMAINS[0]);
        assert_eq!(domains[1], DOMAINS[1]);
        assert_eq!(domains[2], DOMAINS[2]);
        assert_eq!(domains[3], DOMAINS[3]);
        assert_eq!(domains[4], DOMAINS[4]);
        assert_eq!(domains[5], DOMAINS[5]);
        assert_eq!(domains[6], DOMAINS[6]);
        assert_eq!(domains[7], DOMAINS[7]);
        assert_eq!(domains[8], DOMAINS[8]);
        assert_eq!(domains[9], DOMAINS[9]);
        assert_eq!(domains[10], DOMAINS[10]);
        assert_eq!(domains[11], DOMAINS[11]);
        assert_eq!(domains[12], DOMAINS[12], "domains[12] is wrong");
        assert_eq!(domains[13], DOMAINS[13], "domains[13] is wrong");
        assert_eq!(domains[14], DOMAINS[14], "domains[14] is wrong");
        assert_eq!(domains[15], DOMAINS[15], "domains[15] is wrong");
        assert_eq!(domains[16], DOMAINS[16], "domains[16] is wrong");
        assert_eq!(domains[17], DOMAINS[17], "domains[17] is wrong");
        assert_eq!(domains[18], DOMAINS[18], "domains[18] is wrong");
        assert_eq!(domains[19], DOMAINS[19], "domains[19] is wrong");
        assert_eq!(domains[20], DOMAINS[20], "domains[20] is wrong");
        assert_eq!(domains[21], DOMAINS[21], "domains[21] is wrong");
        assert_eq!(domains[22], DOMAINS[22], "domains[22] is wrong");
        assert_eq!(domains[23], DOMAINS[23], "domains[23] is wrong");
        assert_eq!(domains[24], DOMAINS[24], "domains[24] is wrong");
        assert_eq!(domains[25], DOMAINS[25], "domains[25] is wrong");
        assert_eq!(domains[26], DOMAINS[26], "domains[26] is wrong");
        assert_eq!(domains[27], DOMAINS[27], "domains[27] is wrong");
        // for (i, domain) in DOMAINS.iter().enumerate() {
        //     assert_eq!(domains[i], *domain, "domain[{}] is wrong", i);
        // }
    }

    #[motsu::test]
    fn test_batch_inverse() {
        let mut denominators = vec![
            DOMAINS[0],
            DOMAINS[3],
            DOMAINS[4],
            DOMAINS[20],
            DOMAINS[21],
            DOMAINS[1],
            DOMAINS[22],
            DOMAINS[2],
            DOMAINS[23],
            DOMAINS[24],
            DOMAINS[15],
            DOMAINS[16],
            DOMAINS[17],
            DOMAINS[19],
            DOMAINS[8],
            DOMAINS[5],
            DOMAINS[10],
            DOMAINS[6],
        ];
        let den_inv = ConstraintPoly::batch_inverse(&mut denominators).unwrap();
        println!("den_inv: {:?}", den_inv);
        assert_eq!(den_inv, DEN_INV);
    }
}
