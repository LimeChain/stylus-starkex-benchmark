// Allow `cargo stylus export-abi` to generate a main function.
// |<------------------- calldata (bytes) ------------------>|
// |       input        | composition_poly |    domains      |
// | 234 x U256 (32B)   |  52 x U256 (32B) | 28 x U256 (32B) |
// | bytes: 7488        | bytes: 1664      | bytes: 928      |
// |--------------------|------------------|-----------------|
// | 0       ...   233  | 234  ...   285   | 286  ...  313   |
// |--------------------|------------------|-----------------|
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::alloy_primitives::{uint, U256};
use stylus_sdk::stylus_core::calls::errors::Error;
use stylus_sdk::{prelude::*};

// debug imports
const PRIME: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256);

#[entrypoint]
#[storage]
pub struct ConstraintPolyFinalizer;
impl ConstraintPolyFinalizer {
    pub fn denominator_invs(domains: &[U256]) -> Result<Vec<U256>, Error> {
        let denominator_idx = [
            0, 3, 4, 20, 21, 1, 22, 2, 23, 24, 15, 16, 17, 19, 8, 5, 10, 6,
        ];
        let mut partial_products = [U256::ZERO; 18];
        let mut prod = U256::from(1);

        // Build partial products
        for (i, idx) in denominator_idx.iter().enumerate() {
            partial_products[i] = prod;
            prod = prod.mul_mod(domains[*idx], PRIME);
        }

        // Compute inverse of the total product
        let mut prod_inv = prod.pow_mod(PRIME.wrapping_sub(U256::from(2)), PRIME);
        if prod_inv.is_zero() {
            return Err(Error::Revert("Batch inverse product is zero.".into()));
        }

        // Compute inverses
        let mut inverses = vec![U256::ZERO; denominator_idx.len()];
        for i in (0..denominator_idx.len()).rev() {
            inverses[i] = partial_products[i].mul_mod(prod_inv, PRIME);
            prod_inv = prod_inv.mul_mod(domains[denominator_idx[i]], PRIME);
        }

        Ok(inverses)
    }
    fn flag_constraint(flag: U256, den_inv: U256, alpha_pow: U256) -> U256 {
        flag.mul_mod(flag, PRIME)
            .add_mod(PRIME.wrapping_sub(flag), PRIME)
            .mul_mod(den_inv, PRIME)
            .mul_mod(alpha_pow, PRIME)
    }
    fn res_val_constraint(res: U256, val: U256, den_inv: U256, alpha_pow: U256) -> U256 {
        res.add_mod(val.mul_mod(den_inv, PRIME).mul_mod(alpha_pow, PRIME), PRIME)
    }
    fn res_val_const_with_sub(
        input_term: U256,
        sub_term: U256,
        den_inv: U256,
        alpha_pow: U256,
        res: U256,
    ) -> U256 {
        res.add_mod(
            input_term
                .add_mod(PRIME.wrapping_sub(sub_term), PRIME)
                .mul_mod(den_inv, PRIME)
                .mul_mod(alpha_pow, PRIME),
            PRIME,
        )
    }
}

#[public]
impl ConstraintPolyFinalizer {
    #[inline]
    pub fn compute(
        &mut self,
        calldata_words: Vec<U256>,
    ) -> Result<U256, Vec<u8>> {
        let input: &[U256] = &calldata_words[..234];
        let composition_poly: &[U256] = &calldata_words[234..286];
        let domains: &[U256] = &calldata_words[286..];

        let den_invs = Self::denominator_invs(&domains)?;
        let mut res: U256 = U256::ZERO;
        let mut val: U256 = U256::ZERO;
        let mut alpha_pows = [U256::ONE; 124];
        for i in 1..124 {
            alpha_pows[i] = alpha_pows[i - 1].mul_mod(input[41], PRIME);
        }
        res = res
            .add_mod(
                Self::flag_constraint(composition_poly[0], domains[3], den_invs[0])
                    .mul_mod(alpha_pows[0], PRIME),
                PRIME,
            )
            .add_mod(
                input[42]
                    .mul_mod(den_invs[1], PRIME)
                    .mul_mod(alpha_pows[1], PRIME),
                PRIME,
            );

        {
            val = input[92] // column3_row1 @ 0xb80
                .add_mod(
                    PRIME.wrapping_sub(
                        input[42] // column0_row0 @ 0x540
                            .mul_mod(input[8], PRIME) // offset_size @ 0x100
                            .add_mod(input[151], PRIME) // column6_row4 @ 0x12e0
                            .mul_mod(input[8], PRIME)
                            .add_mod(input[155], PRIME) // column6_row8 @ 0x1360
                            .mul_mod(input[8], PRIME)
                            .add_mod(input[147], PRIME), // column6_row0 @ 0x1260
                    ),
                    PRIME,
                )
                .mul_mod(den_invs[2], PRIME) // denominator: point^(trace_length / 16) - 1
                // Multiply by alpha^2
                .mul_mod(alpha_pows[2], PRIME);

            // Accumulate result
            res = res.add_mod(val, PRIME);
        }
        // Accumulate into result with current alpha power
        res = res
            .add_mod(
                Self::flag_constraint(composition_poly[4], den_invs[2], alpha_pows[3]),
                PRIME,
            )
            .add_mod(
                Self::flag_constraint(composition_poly[8], den_invs[2], alpha_pows[4]),
                PRIME,
            )
            .add_mod(
                Self::flag_constraint(composition_poly[11], den_invs[2], alpha_pows[5]),
                PRIME,
            )
            .add_mod(
                Self::flag_constraint(composition_poly[14], den_invs[2], alpha_pows[6]),
                PRIME,
            );

        // Constraint expression for cpu/operands/mem_dst_addr:
        // val = input[99] + input[9] - (composition_poly[0] * input[200] + (1 - composition_poly[0]) * input[194] + input[147])
        val = input[99].add_mod(input[9], PRIME).add_mod(
            PRIME.wrapping_sub(
                composition_poly[0]
                    .mul_mod(input[200], PRIME)
                    .add_mod(
                        U256::ONE
                            .add_mod(PRIME.wrapping_sub(composition_poly[0]), PRIME)
                            .mul_mod(input[194], PRIME),
                        PRIME,
                    )
                    .add_mod(input[147], PRIME),
            ),
            PRIME,
        );

        // res += val * alpha^7
        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[7]);

        val = input[95].add_mod(input[9], PRIME).add_mod(
            PRIME.wrapping_sub(
                composition_poly[15]
                    .mul_mod(input[200], PRIME)
                    .add_mod(
                        U256::ONE
                            .add_mod(PRIME.wrapping_sub(composition_poly[15]), PRIME)
                            .mul_mod(input[194], PRIME),
                        PRIME,
                    )
                    .add_mod(input[155], PRIME),
            ),
            PRIME,
        );

        // res += val * alpha^8
        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[8]);

        val = input[103].add_mod(input[9], PRIME).add_mod(
            PRIME.wrapping_sub(
                composition_poly[1]
                    .mul_mod(input[91], PRIME)
                    .add_mod(composition_poly[2].mul_mod(input[194], PRIME), PRIME)
                    .add_mod(composition_poly[3].mul_mod(input[200], PRIME), PRIME)
                    .add_mod(composition_poly[4].mul_mod(input[96], PRIME), PRIME)
                    .add_mod(input[151], PRIME),
            ),
            PRIME,
        );

        // res += val * alpha^9
        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[9]);

        val = input[197].add_mod(
            PRIME.wrapping_sub(input[96].mul_mod(input[104], PRIME)),
            PRIME,
        );

        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[10]);

        // cpu/operands/res
        // (1 - bit_9) * col8_row12 - (bit_5 * (col3_row5 + col3_row13) + bit_6 * col8_row4 + flag_res_op1_0 * col3_row13)
        val = U256::ONE
            .add_mod(PRIME.wrapping_sub(composition_poly[7]), PRIME)
            .mul_mod(input[203], PRIME)
            .add_mod(
                PRIME.wrapping_sub(
                    composition_poly[5]
                        .mul_mod(input[96].add_mod(input[104], PRIME), PRIME)
                        .add_mod(composition_poly[6].mul_mod(input[197], PRIME), PRIME)
                        .add_mod(composition_poly[8].mul_mod(input[104], PRIME), PRIME),
                ),
                PRIME,
            );

        // res += val * alpha ** 11.
        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[11]);

        // Constraint: col8_row2 - bit_9 * col3_row9
        val = input[196]
            .add_mod(
                PRIME.wrapping_sub(composition_poly[7].mul_mod(input[100], PRIME)),
                PRIME,
            )
            .mul_mod(domains[20], PRIME);
        // res += val * alpha ** 12.
        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[12]);
        {
            // Constraint: column8_row10 - column8_row2 * column8_row12
            val = input[202]
                .add_mod(
                    PRIME.wrapping_sub(input[196].mul_mod(input[203], PRIME)),
                    PRIME,
                )
                .mul_mod(domains[20], PRIME);

            // res += val * alpha^13
            res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[13]);
        }
        {
            // Intermediate values
            let one_minus_bit_9 = U256::ONE.add_mod(PRIME.wrapping_sub(composition_poly[7]), PRIME); // 0x1e20

            let diff_16_minus_sum_0_13 = input[105].add_mod(
                // 0xd20
                PRIME.wrapping_sub(
                    input[91].add_mod(input[104], PRIME), // input[91] = 0xb60, input[104] = 0xd00
                ),
                PRIME,
            );

            let left = one_minus_bit_9
                .mul_mod(input[105], PRIME) // 0xd20
                .add_mod(
                    input[196].mul_mod(diff_16_minus_sum_0_13, PRIME), // input[196] = 0x1880
                    PRIME,
                );

            let right = composition_poly[11]
                .mul_mod(composition_poly[16], PRIME) // 0x1ea0 * 0x1f40
                .add_mod(
                    composition_poly[9].mul_mod(input[203], PRIME), // 0x1e60 * 0x1960
                    PRIME,
                )
                .add_mod(
                    composition_poly[10].mul_mod(
                        input[91].add_mod(input[203], PRIME), // 0xb60 + 0x1960
                        PRIME,
                    ),
                    PRIME,
                );

            res = Self::res_val_constraint(
                res,
                left.add_mod(PRIME.wrapping_sub(right), PRIME)
                    .mul_mod(domains[20], PRIME),
                den_invs[2],
                alpha_pows[14],
            );
        }

        // res += val * alpha^15
        res = res.add_mod(
            input[202]
                .add_mod(PRIME.wrapping_sub(composition_poly[7]), PRIME)
                .mul_mod(
                    input[105].add_mod(PRIME.wrapping_sub(composition_poly[16]), PRIME),
                    PRIME,
                )
                .mul_mod(domains[20], PRIME)
                .mul_mod(den_invs[2], PRIME)
                .mul_mod(alpha_pows[15], PRIME),
            PRIME,
        );
        {
            // val = column8_row16 - (column8_row0 + bit10 * column8_row12 + bit11 + bit12 * 2)
            let term = input[194]
                .add_mod(composition_poly[17].mul_mod(input[203], PRIME), PRIME)
                .add_mod(composition_poly[18], PRIME)
                .add_mod(composition_poly[12].mul_mod(U256::from(2), PRIME), PRIME);

            let val = input[206]
                .add_mod(PRIME.wrapping_sub(term), PRIME)
                .mul_mod(domains[20], PRIME)
                .mul_mod(den_invs[2], PRIME);

            res = res.add_mod(val.mul_mod(alpha_pows[16], PRIME), PRIME);
        }

        // val = column8_row24 - (fp_update_regular_0 * column8_row8 + bit13 * column3_row9 + bit12 * (column8_row0 + 2))
        res = res.add_mod(
            input[209]
                .add_mod(
                    PRIME.wrapping_sub(
                        composition_poly[14]
                            .mul_mod(input[200], PRIME)
                            .add_mod(composition_poly[13].mul_mod(input[100], PRIME), PRIME)
                            .add_mod(
                                composition_poly[12]
                                    .mul_mod(input[194].add_mod(U256::from(2), PRIME), PRIME),
                                PRIME,
                            ),
                    ),
                    PRIME,
                )
                .mul_mod(domains[20], PRIME)
                .mul_mod(den_invs[2], PRIME)
                .mul_mod(alpha_pows[17], PRIME),
            PRIME,
        );

        // val = bit_12 * (column3_row9 - column8_row8)

        res = Self::res_val_constraint(
            res,
            composition_poly[12].mul_mod(
                input[100].add_mod(PRIME.wrapping_sub(input[200]), PRIME),
                PRIME,
            ),
            den_invs[2],
            alpha_pows[18],
        );

        // val = bit_12 * (column3_row5 - (column3_row0 + bit_2 + 1))

        val = composition_poly[12].mul_mod(
            input[96].add_mod(
                PRIME.wrapping_sub(
                    input[91]
                        .add_mod(composition_poly[1], PRIME)
                        .add_mod(U256::ONE, PRIME),
                ),
                PRIME,
            ),
            PRIME,
        );

        // res += val * alpha ** 19.
        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[19]);

        val = composition_poly[12].mul_mod(
            input[147].add_mod(PRIME.wrapping_sub(input[9]), PRIME),
            PRIME,
        );

        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[20]);

        val = composition_poly[12].mul_mod(
            input[155].add_mod(
                PRIME.wrapping_sub(input[9].add_mod(U256::ONE, PRIME)),
                PRIME,
            ),
            PRIME,
        );

        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[21]);

        val = composition_poly[12].mul_mod(
            composition_poly[12]
                .add_mod(composition_poly[12], PRIME)
                .add_mod(U256::from(1), PRIME)
                .add_mod(U256::from(1), PRIME)
                .add_mod(
                    PRIME.wrapping_sub(
                        composition_poly[0]
                            .add_mod(composition_poly[15], PRIME)
                            .add_mod(U256::from(4), PRIME),
                    ),
                    PRIME,
                ),
            PRIME,
        );
        // res += val * alpha ** 22.
        res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[22]);

        res = Self::res_val_constraint(
            res,
            composition_poly[13].mul_mod(
                input[147]
                    .add_mod(U256::from(2), PRIME)
                    .add_mod(PRIME.wrapping_sub(input[9]), PRIME),
                PRIME,
            ),
            den_invs[2],
            alpha_pows[23],
        );

        {
            let val = composition_poly[13].mul_mod(
                input[151]
                    .add_mod(U256::from(1), PRIME)
                    .add_mod(PRIME.wrapping_sub(input[9]), PRIME),
                PRIME,
            );

            res = Self::res_val_constraint(res, val, den_invs[2], alpha_pows[24]);
        }
        {
            let sum = composition_poly[9]
                .add_mod(composition_poly[0], PRIME)
                .add_mod(composition_poly[3], PRIME)
                .add_mod(composition_poly[8], PRIME)
                .add_mod(PRIME.wrapping_sub(U256::from(4)), PRIME);

            res = Self::res_val_constraint(
                res,
                composition_poly[13].mul_mod(sum, PRIME),
                den_invs[2],
                alpha_pows[25],
            );
        }

        res = Self::res_val_constraint(
            res,
            composition_poly[19].mul_mod(
                input[100].add_mod(PRIME.wrapping_sub(input[203]), PRIME),
                PRIME,
            ),
            den_invs[2],
            alpha_pows[26],
        );

        res = Self::res_val_constraint(
            res,
            input[194].add_mod(PRIME.wrapping_sub(input[10]), PRIME),
            den_invs[4],
            alpha_pows[27],
        );

        res = Self::res_val_constraint(
            res,
            input[200].add_mod(PRIME.wrapping_sub(input[10]), PRIME),
            den_invs[4],
            alpha_pows[28],
        );

        {
            res = res.add_mod(
                input[91]
                    .add_mod(PRIME.wrapping_sub(input[11]), PRIME)
                    .mul_mod(den_invs[4], PRIME)
                    .mul_mod(alpha_pows[29], PRIME),
                PRIME,
            );
        }

        // res += val * alpha ** 30.
        res = res.add_mod(
            input[194]
                .add_mod(PRIME.wrapping_sub(input[12]), PRIME)
                .mul_mod(den_invs[3], PRIME)
                .mul_mod(alpha_pows[30], PRIME),
            PRIME,
        );

        // res += val * alpha ** 31.
        res = res.add_mod(
            input[200]
                .add_mod(PRIME.wrapping_sub(input[10]), PRIME)
                .mul_mod(den_invs[3], PRIME)
                .mul_mod(alpha_pows[31], PRIME),
            PRIME,
        );

        // res += val * alpha ** 32.
        res = res.add_mod(
            input[91]
                .add_mod(PRIME.wrapping_sub(input[13]), PRIME)
                .mul_mod(den_invs[3], PRIME)
                .mul_mod(alpha_pows[32], PRIME),
            PRIME,
        );
        {
            let term1 = input[14].add_mod(
                PRIME.wrapping_sub(input[133].add_mod(input[15].mul_mod(input[134], PRIME), PRIME)),
                PRIME,
            );

            let val = term1
                .mul_mod(input[230], PRIME)
                .add_mod(input[91], PRIME)
                .add_mod(input[15].mul_mod(input[92], PRIME), PRIME)
                .add_mod(PRIME.wrapping_sub(input[14]), PRIME);

            // res += val * alpha ** 33.
            res = Self::res_val_constraint(res, val, den_invs[4], alpha_pows[33]);
        }
        {
            let term1 = input[14].add_mod(
                PRIME.wrapping_sub(input[135].add_mod(input[15].mul_mod(input[136], PRIME), PRIME)),
                PRIME,
            );

            let term2 = input[14].add_mod(
                PRIME.wrapping_sub(input[93].add_mod(input[15].mul_mod(input[94], PRIME), PRIME)),
                PRIME,
            );

            let val = term1
                .mul_mod(input[232], PRIME)
                .add_mod(PRIME.wrapping_sub(term2.mul_mod(input[230], PRIME)), PRIME);

            res = Self::res_val_constraint(
                res,
                val.mul_mod(domains[22], PRIME),
                den_invs[5],
                alpha_pows[34],
            );
        }
        {
            let val = input[230].add_mod(PRIME.wrapping_sub(input[16]), PRIME);

            res = Self::res_val_constraint(res, val, den_invs[6], alpha_pows[35]);
        }
        {
            val = composition_poly[20]
                .mul_mod(composition_poly[20], PRIME)
                .add_mod(PRIME.wrapping_sub(composition_poly[20]), PRIME)
                .mul_mod(domains[22], PRIME);
            // res += val * alpha ** 36.
            // res = res.add_mod(val.mul_mod(alpha_pows[36], PRIME), PRIME);
            res = Self::res_val_constraint(res, val, den_invs[5], alpha_pows[36]);
        }
        {
            //06185fe9960b9c13158b8394c6428cb4c4957440fb05f151aa2b53de8a57367d
            let val = composition_poly[20]
                .add_mod(PRIME.wrapping_sub(U256::from(1)), PRIME)
                .mul_mod(
                    input[134].add_mod(PRIME.wrapping_sub(input[136]), PRIME),
                    PRIME,
                )
                .mul_mod(domains[22], PRIME)
                .mul_mod(den_invs[5], PRIME);
            // res += val * alpha ** 37.
            res = res.add_mod(val.mul_mod(alpha_pows[37], PRIME), PRIME);
        }

        // res += val * alpha ** 38.

        res = Self::res_val_constraint(
            Self::res_val_constraint(
                res,
                input[133].add_mod(PRIME.wrapping_sub(U256::from(1)), PRIME),
                den_invs[4],
                alpha_pows[38],
            ),
            input[93],
            den_invs[2],
            alpha_pows[39],
        );

        res = Self::res_val_constraint(res, input[94], den_invs[2], alpha_pows[40]);

        {
            let val = input[17]
                .add_mod(PRIME.wrapping_sub(input[149]), PRIME)
                .mul_mod(input[231], PRIME)
                .add_mod(input[147], PRIME)
                .add_mod(PRIME.wrapping_sub(input[17]), PRIME)
                .mul_mod(den_invs[4], PRIME);

            res = res.add_mod(val.mul_mod(alpha_pows[41], PRIME), PRIME);
        }
        {
            let lhs = input[17]
                .add_mod(PRIME.wrapping_sub(input[153]), PRIME)
                .mul_mod(input[233], PRIME);

            let rhs = input[17]
                .add_mod(PRIME.wrapping_sub(input[151]), PRIME)
                .mul_mod(input[231], PRIME);

            let val = lhs
                .add_mod(PRIME.wrapping_sub(rhs), PRIME)
                .mul_mod(domains[23], PRIME);

            res = Self::res_val_constraint(res, val, den_invs[7], alpha_pows[42]);
        }

        res = Self::res_val_constraint(
            res,
            input[231].add_mod(PRIME.wrapping_sub(input[18]), PRIME),
            den_invs[8],
            alpha_pows[43],
        );

        // let diff = composition_poly[21]; // 0x1fe0 = 255

        res = res.add_mod(
            Self::flag_constraint(composition_poly[21], domains[23], den_invs[7])
                .mul_mod(alpha_pows[44], PRIME),
            PRIME,
        );

        {
            let val = input[149] // column6_row2
                .add_mod(PRIME.wrapping_sub(input[19]), PRIME) // range_check_min
                .mul_mod(den_invs[4], PRIME);
            // res += val * alpha ** 45.
            res = res.add_mod(val.mul_mod(alpha_pows[45], PRIME), PRIME);
        }
        {
            let val = input[149]
                .add_mod(PRIME.wrapping_sub(input[20]), PRIME)
                .mul_mod(den_invs[8], PRIME);

            res = res.add_mod(val.mul_mod(alpha_pows[46], PRIME), PRIME);
        }
        {
            let val = input[21]
                .add_mod(PRIME.wrapping_sub(input[89]), PRIME)
                .mul_mod(input[228], PRIME)
                .add_mod(input[58], PRIME)
                .add_mod(PRIME.wrapping_sub(input[21]), PRIME);

            res = Self::res_val_constraint(res, val, den_invs[4], alpha_pows[47]);
        }
        {
            let lhs = input[21]
                .add_mod(PRIME.wrapping_sub(input[90]), PRIME)
                .mul_mod(input[229], PRIME);

            let rhs = input[21]
                .add_mod(PRIME.wrapping_sub(input[59]), PRIME)
                .mul_mod(input[228], PRIME);

            let val = lhs
                .add_mod(PRIME.wrapping_sub(rhs), PRIME)
                .mul_mod(domains[24], PRIME);

            res = Self::res_val_constraint(res, val, den_invs[0], alpha_pows[48]);
        }

        // let val = input[226]
        //     .add_mod(PRIME.wrapping_sub(U256::ONE), PRIME)
        //     .mul_mod(den_invs[4], PRIME);

        // res = res.add_mod(val.mul_mod(alpha_pows[50], PRIME), PRIME);

        // res = res.add_mod(val.mul_mod(alpha_pows[51], PRIME), PRIME);
        res = Self::res_val_const_with_sub(
            input[89],
            input[23],
            den_invs[4],
            alpha_pows[51],
            Self::res_val_const_with_sub(
                input[226],
                U256::ONE,
                den_invs[4],
                alpha_pows[50],
                Self::res_val_const_with_sub(
                    input[228],
                    input[22],
                    den_invs[9],
                    alpha_pows[49],
                    res,
                ),
            ),
        );

        {
            let diff = input[90].add_mod(PRIME.wrapping_sub(input[89]), PRIME);

            let sub_term = PRIME.wrapping_sub(
                input[226]
                    .mul_mod(
                        U256::ONE.add_mod(input[24].mul_mod(diff, PRIME), PRIME),
                        PRIME,
                    )
                    .add_mod(input[25].mul_mod(diff, PRIME).mul_mod(diff, PRIME), PRIME),
            );
            let val = input[227]
                .add_mod(sub_term, PRIME)
                .mul_mod(domains[24], PRIME);
            // res += val * alpha ** 52.
            res = Self::res_val_constraint(res, val, den_invs[0], alpha_pows[52]);
        }

        res = Self::res_val_const_with_sub(input[226], input[26], den_invs[9], alpha_pows[53], res);

        {
            let val = input[185].mul_mod(
                input[169].add_mod(
                    PRIME.wrapping_sub(input[173].add_mod(input[173], PRIME)),
                    PRIME,
                ),
                PRIME,
            );

            res = Self::res_val_constraint(res, val, den_invs[10], alpha_pows[54]);
        }
        {
            let val = input[185]
                .mul_mod(
                    input[173].add_mod(
                        PRIME.wrapping_sub(
                            uint!(3138550867693340381917894711603833208051177722232017256448_U256)
                                .mul_mod(input[186], PRIME),
                        ),
                        PRIME,
                    ),
                    PRIME,
                )
                .mul_mod(den_invs[10], PRIME);
            // res += val * alpha ** 55.
            res = res.add_mod(val.mul_mod(alpha_pows[55], PRIME), PRIME);
        }

        res = Self::res_val_const_with_sub(
            input[185],
            input[192].mul_mod(
                input[186].add_mod(
                    PRIME.wrapping_sub(input[187].add_mod(input[187], PRIME)),
                    PRIME,
                ),
                PRIME,
            ),
            den_invs[10],
            alpha_pows[56],
            res,
        );

        {
            let val = input[192].mul_mod(
                input[187].add_mod(
                    PRIME.wrapping_sub(U256::from(8).mul_mod(input[188], PRIME)),
                    PRIME,
                ),
                PRIME,
            );

            res = Self::res_val_constraint(res, val, den_invs[10], alpha_pows[57]);
        }
        {
            let diff_1004_1008 = input[190].add_mod(
                PRIME.wrapping_sub(input[191].add_mod(input[191], PRIME)),
                PRIME,
            );
            let diff_784_788 = input[188].add_mod(
                PRIME.wrapping_sub(input[189].add_mod(input[189], PRIME)),
                PRIME,
            );

            res = Self::res_val_const_with_sub(
                input[192],
                diff_1004_1008.mul_mod(diff_784_788, PRIME),
                den_invs[10],
                alpha_pows[58],
                res,
            );
        }
        {
            let val = input[190]
                .add_mod(
                    PRIME.wrapping_sub(input[191].add_mod(input[191], PRIME)),
                    PRIME,
                )
                .mul_mod(
                    input[189].add_mod(
                        PRIME.wrapping_sub(
                            U256::from(18014398509481984u64).mul_mod(input[190], PRIME),
                        ),
                        PRIME,
                    ),
                    PRIME,
                );
            // res += val * alpha ** 60.
            res = Self::res_val_constraint(res, val, den_invs[10], alpha_pows[59]);
        }

        // res += val * alpha ** 60.
        res = res.add_mod(
            composition_poly[22]
                .mul_mod(
                    composition_poly[22].add_mod(PRIME.wrapping_sub(U256::ONE), PRIME),
                    PRIME,
                )
                .mul_mod(domains[16], PRIME)
                .mul_mod(den_invs[7], PRIME)
                .mul_mod(alpha_pows[60], PRIME),
            PRIME,
        );

        res = Self::res_val_constraint(
            Self::res_val_constraint(res, input[169], den_invs[12], alpha_pows[61]),
            input[169],
            den_invs[11],
            alpha_pows[62],
        );

        {
            // res += val * alpha ** 63.
            let val = composition_poly[22]
                .mul_mod(
                    input[150].add_mod(PRIME.wrapping_sub(input[1]), PRIME),
                    PRIME,
                )
                .add_mod(
                    PRIME.wrapping_sub(input[171].mul_mod(
                        input[148].add_mod(PRIME.wrapping_sub(input[0]), PRIME),
                        PRIME,
                    )),
                    PRIME,
                )
                .mul_mod(domains[16], PRIME);
            // res = res.add_mod(val.mul_mod(alpha_pows[63], PRIME)    , PRIME);
            res = Self::res_val_constraint(res, val, den_invs[7], alpha_pows[63]);
        }
        {
            // res += val * alpha ** 64.
            let lhs = input[171].mul_mod(input[171], PRIME);
            let rhs = composition_poly[22].mul_mod(
                input[148]
                    .add_mod(input[0], PRIME)
                    .add_mod(input[152], PRIME),
                PRIME,
            );
            let val = lhs
                .add_mod(PRIME.wrapping_sub(rhs), PRIME)
                .mul_mod(domains[16], PRIME);
            res = Self::res_val_constraint(res, val, den_invs[7], alpha_pows[64]);
        }
        {
            // res += val * alpha ** 65.
            let lhs = composition_poly[22].mul_mod(input[150].add_mod(input[154], PRIME), PRIME);
            let rhs = input[171].mul_mod(
                input[148].add_mod(PRIME.wrapping_sub(input[152]), PRIME),
                PRIME,
            );
            let val = lhs
                .add_mod(PRIME.wrapping_sub(rhs), PRIME)
                .mul_mod(domains[16], PRIME);
            res = Self::res_val_constraint(res, val, den_invs[7], alpha_pows[65]);
        }
        {
            // res += val * alpha ** 66.
            let val = composition_poly[23]
                .mul_mod(
                    input[152].add_mod(PRIME.wrapping_sub(input[148]), PRIME),
                    PRIME,
                )
                .mul_mod(domains[16], PRIME);
            res = Self::res_val_constraint(res, val, den_invs[7], alpha_pows[66]);
        }
        {
            // res += val * alpha ** 67.
            let val = composition_poly[23]
                .mul_mod(
                    input[154].add_mod(PRIME.wrapping_sub(input[150]), PRIME),
                    PRIME,
                )
                .mul_mod(domains[16], PRIME);
            res = Self::res_val_constraint(res, val, den_invs[7], alpha_pows[67]);
        }
        {
            // res += val * alpha ** 68.
            let val = input[166]
                .add_mod(PRIME.wrapping_sub(input[164]), PRIME)
                .mul_mod(domains[18], PRIME);
            res = Self::res_val_constraint(res, val, den_invs[10], alpha_pows[68]);
        }
        {
            // res += val * alpha ** 69.
            let val = input[167]
                .add_mod(PRIME.wrapping_sub(input[165]), PRIME)
                .mul_mod(domains[18], PRIME);
            res = Self::res_val_constraint(res, val, den_invs[10], alpha_pows[69]);
        }

        res = Self::res_val_const_with_sub(
            input[102],
            input[169],
            den_invs[13],
            alpha_pows[72],
            Self::res_val_const_with_sub(
                input[150],
                input[28],
                den_invs[13],
                alpha_pows[71],
                Self::res_val_const_with_sub(
                    input[148],
                    input[27],
                    den_invs[13],
                    alpha_pows[70],
                    res,
                ),
            ),
        );
        {
            // res += val * alpha ** 73.
            let val = input[132]
                .add_mod(
                    PRIME.wrapping_sub(input[128].add_mod(U256::ONE, PRIME)),
                    PRIME,
                )
                .mul_mod(domains[25], PRIME);
            // res = res.add_mod(val.mul_mod(alpha_pows[73], PRIME), PRIME);
            res = Self::res_val_constraint(res, val, den_invs[13], alpha_pows[73]);
        }

        // res += val * alpha ** 75.
        // let val = input[131]
        //     .add_mod(PRIME.wrapping_sub(input[193]), PRIME);
        // .mul_mod(den_invs[13], PRIME);
        // res = res.add_mod(val.mul_mod(alpha_pows[75], PRIME), PRIME);
        res = Self::res_val_const_with_sub(
            input[131],
            input[193],
            den_invs[13],
            alpha_pows[75],
            Self::res_val_const_with_sub(input[101], input[29], den_invs[4], alpha_pows[74], res),
        );

        // res += val * alpha ** 77.
        // let val = input[129]
        //     .add_mod(PRIME.wrapping_sub(input[168]), PRIME)
        //     .mul_mod(den_invs[13], PRIME);
        // res = res.add_mod(val.mul_mod(alpha_pows[77], PRIME), PRIME);
        res = Self::res_val_const_with_sub(
            input[129],
            input[168],
            den_invs[13],
            alpha_pows[77],
            Self::res_val_const_with_sub(
                input[130],
                input[101].add_mod(U256::ONE, PRIME),
                den_invs[13],
                alpha_pows[76],
                res,
            ),
        );

        res = Self::res_val_const_with_sub(
            composition_poly[31],
            input[118],
            den_invs[14],
            alpha_pows[79],
            Self::res_val_const_with_sub(
                input[128],
                input[130].add_mod(U256::ONE, PRIME),
                den_invs[13],
                alpha_pows[78],
                res,
            ),
        );

        {
            // res += val * alpha ** 80.
            let val = input[127]
                .add_mod(
                    PRIME.wrapping_sub(input[117].add_mod(U256::ONE, PRIME)),
                    PRIME,
                )
                .mul_mod(domains[26], PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[80], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 81.
            let val = input[117]
                .add_mod(PRIME.wrapping_sub(input[30]), PRIME)
                .mul_mod(den_invs[4], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[81], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 82.
            let val = input[108]
                .add_mod(PRIME.wrapping_sub(input[31]), PRIME)
                .mul_mod(den_invs[4], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[82], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 83.
            let val = input[114]
                .add_mod(
                    PRIME.wrapping_sub(input[108].add_mod(U256::ONE, PRIME)),
                    PRIME,
                )
                .mul_mod(domains[9], PRIME)
                .mul_mod(den_invs[15], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[83], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 84.
            let val = input[112]
                .add_mod(
                    PRIME.wrapping_sub(input[124].add_mod(U256::ONE, PRIME)),
                    PRIME,
                )
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[84], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 85.
            let val = input[126]
                .add_mod(
                    PRIME.wrapping_sub(input[112].add_mod(U256::ONE, PRIME)),
                    PRIME,
                )
                .mul_mod(domains[26], PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[85], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 86.
            let val = composition_poly[32]
                .add_mod(composition_poly[33], PRIME)
                .add_mod(PRIME.wrapping_sub(input[109]), PRIME)
                .mul_mod(den_invs[15], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[86], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 87.
            let val = input[113]
                .add_mod(
                    PRIME.wrapping_sub(input[121].add_mod(input[125], PRIME)),
                    PRIME,
                )
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[87], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 88.
            let val = input[58]
                .add_mod(input[75], PRIME)
                .add_mod(
                    PRIME.wrapping_sub(
                        input[83].add_mod(input[77].add_mod(input[77], PRIME), PRIME),
                    ),
                    PRIME,
                )
                .mul_mod(den_invs[16], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[88], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 89.
            let val = (input[79]
                .add_mod(input[85], PRIME)
                .mul_mod(U256::from(16), PRIME))
            .add_mod(PRIME.wrapping_sub(input[59]), PRIME)
            .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[89], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 90.
            let val = (input[80] + input[86])
                .mul_mod(U256::from(16), PRIME)
                .add_mod(PRIME.wrapping_sub(input[78]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[90], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 91.
            let val = (input[81] + input[87])
                .mul_mod(U256::from(16), PRIME)
                .add_mod(PRIME.wrapping_sub(input[76]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[91], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 92.
            let val = (input[82] + input[88])
                .mul_mod(U256::from(256), PRIME)
                .add_mod(PRIME.wrapping_sub(input[84]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[92], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 93.
            let val = input[97]
                .add_mod(PRIME.wrapping_sub(input[32]), PRIME)
                .mul_mod(den_invs[4], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[93], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 94.
            let val = input[115]
                .add_mod(
                    PRIME.wrapping_sub(input[97].add_mod(U256::from(3), PRIME)),
                    PRIME,
                )
                .mul_mod(domains[27], PRIME)
                .mul_mod(den_invs[17], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[94], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 95.
            let val = input[110]
                .add_mod(
                    PRIME.wrapping_sub(input[32].add_mod(U256::ONE, PRIME)),
                    PRIME,
                )
                .mul_mod(den_invs[4], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[95], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 96.
            let val = input[122]
                .add_mod(
                    PRIME.wrapping_sub(input[110].add_mod(U256::from(3), PRIME)),
                    PRIME,
                )
                .mul_mod(domains[27], PRIME)
                .mul_mod(den_invs[17], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[96], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 97.
            let val = input[106]
                .add_mod(
                    PRIME.wrapping_sub(input[32].add_mod(U256::from(2), PRIME)),
                    PRIME,
                )
                .mul_mod(den_invs[4], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[97], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 98.
            let val = input[119]
                .add_mod(
                    PRIME.wrapping_sub(input[106].add_mod(U256::from(3), PRIME)),
                    PRIME,
                )
                .mul_mod(domains[27], PRIME)
                .mul_mod(den_invs[17], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[98], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 99.
            let val = input[199]
                .mul_mod(input[199], PRIME)
                .add_mod(PRIME.wrapping_sub(input[201]), PRIME)
                .mul_mod(den_invs[2], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[99], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 100.
            let val = input[205]
                .mul_mod(input[205], PRIME)
                .add_mod(PRIME.wrapping_sub(input[198]), PRIME)
                .mul_mod(den_invs[2], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[100], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 101.
            let val = input[195]
                .mul_mod(input[195], PRIME)
                .add_mod(PRIME.wrapping_sub(input[204]), PRIME)
                .mul_mod(den_invs[2], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[101], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 102.
            let val = input[137]
                .mul_mod(input[137], PRIME)
                .add_mod(PRIME.wrapping_sub(input[138]), PRIME)
                .mul_mod(den_invs[5], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[102], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 103.
            let val = input[170]
                .mul_mod(input[170], PRIME)
                .add_mod(PRIME.wrapping_sub(input[172]), PRIME)
                .mul_mod(domains[12], PRIME)
                .mul_mod(den_invs[7], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[103], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 104.
            let val = input[98]
                .add_mod(
                    uint!(2950795762459345168613727575620414179244544320470208355568817838579231751791_U256),
                    PRIME,
                )
                .add_mod(PRIME.wrapping_sub(input[199]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[104], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 105.
            let val = input[111]
                .add_mod(
                    uint!(1587446564224215276866294500450702039420286416111469274423465069420553242820_U256),
                    PRIME,
                )
                .add_mod(PRIME.wrapping_sub(input[205]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[105], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 106.
            let val = input[107]
                .add_mod(
                    uint!(1645965921169490687904413452218868659025437693527479459426157555728339600137_U256),
                    PRIME,
                )
                .add_mod(PRIME.wrapping_sub(input[195]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[106], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 107.
            let val = input[208]
                .add_mod(
                    PRIME.wrapping_sub(
                        composition_poly[34]
                            .add_mod(composition_poly[34], PRIME)
                            .add_mod(composition_poly[34], PRIME)
                            .add_mod(composition_poly[35], PRIME)
                            .add_mod(composition_poly[36], PRIME)
                            .add_mod(input[2], PRIME),
                    ),
                    PRIME,
                )
                .mul_mod(domains[7], PRIME)
                .mul_mod(den_invs[2], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[107], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 108.
            let val = input[210]
                .add_mod(composition_poly[35], PRIME)
                .add_mod(
                    PRIME.wrapping_sub(
                        composition_poly[34]
                            .add_mod(composition_poly[36], PRIME)
                            .add_mod(input[3], PRIME),
                    ),
                    PRIME,
                )
                .mul_mod(domains[7], PRIME)
                .mul_mod(den_invs[2], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[108], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 109.
            let val = input[207]
                .add_mod(composition_poly[36], PRIME)
                .add_mod(composition_poly[36], PRIME)
                .add_mod(
                    PRIME.wrapping_sub(
                        composition_poly[34]
                            .add_mod(composition_poly[35], PRIME)
                            .add_mod(input[4], PRIME),
                    ),
                    PRIME,
                )
                .mul_mod(domains[7], PRIME)
                .mul_mod(den_invs[2], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[109], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 110.
            let val: U256 = input[116]
                .add_mod(
                    PRIME.wrapping_sub(
                        composition_poly[37]
                            .add_mod(composition_poly[37], PRIME)
                            .add_mod(composition_poly[37], PRIME)
                            .add_mod(composition_poly[38], PRIME)
                            .add_mod(composition_poly[39], PRIME),
                    ),
                    PRIME,
                )
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[110], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 111.
            let val = input[123]
                .add_mod(composition_poly[38], PRIME)
                .add_mod(
                    PRIME.wrapping_sub(composition_poly[37].add_mod(composition_poly[39], PRIME)),
                    PRIME,
                )
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[111], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 112.
            let val = input[120]
                .add_mod(composition_poly[39], PRIME)
                .add_mod(composition_poly[39], PRIME)
                .add_mod(
                    PRIME.wrapping_sub(composition_poly[37].add_mod(composition_poly[38], PRIME)),
                    PRIME,
                )
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[112], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 113.
            let val = input[144]
                .add_mod(PRIME.wrapping_sub(input[170]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[113], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 114.
            let val = input[145]
                .add_mod(PRIME.wrapping_sub(input[174]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[114], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 115.
            let val = input[146]
                .add_mod(PRIME.wrapping_sub(input[176]), PRIME)
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[115], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 116.
            let val = input[137]
                .add_mod(composition_poly[42], PRIME)
                .add_mod(composition_poly[42], PRIME)
                .add_mod(
                    PRIME.wrapping_sub(
                        composition_poly[40].add_mod(composition_poly[41], PRIME)
                            .add_mod(
                                uint!(2121140748740143694053732746913428481442990369183417228688865837805149503386_U256),
                                PRIME,
                            )
                    ),
                    PRIME,
                )
                .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[116], PRIME), PRIME);
        }

        {
            // res += val * alpha ** 117.
            let val = input[139]
                .add_mod(
                    PRIME - (
                        uint!(3618502788666131213697322783095070105623107215331596699973092056135872020477_U256)
                            .mul_mod(composition_poly[41], PRIME)
                            .add_mod(
                                U256::from(10).mul_mod(composition_poly[42], PRIME),
                                PRIME,
                            )
                            .add_mod(
                                U256::from(4).mul_mod(input[137], PRIME),
                                PRIME,
                            )
                            .add_mod(
                                uint!(3618502788666131213697322783095070105623107215331596699973092056135872020479_U256)
                                    .mul_mod(composition_poly[43], PRIME),
                                PRIME,
                            )
                            .add_mod(
                                uint!(2006642341318481906727563724340978325665491359415674592697055778067937914672_U256),
                                PRIME,
                            )
                    ),
                    PRIME,
                )
                .mul_mod(den_invs[14], PRIME);

            res = res.add_mod(val.mul_mod(alpha_pows[117], PRIME), PRIME);
        }
        {
            // res += val * alpha ** 118.
            let val = input[141].add_mod(
                PRIME.wrapping_sub(
                    uint!(8_U256).mul_mod(composition_poly[42], PRIME)
                        .add_mod(uint!(4_U256).mul_mod(input[137], PRIME), PRIME)
                        .add_mod(uint!(6_U256).mul_mod(composition_poly[43], PRIME), PRIME)
                        .add_mod(input[139], PRIME)
                        .add_mod(input[139], PRIME)
                        .add_mod(
                            uint!(3618502788666131213697322783095070105623107215331596699973092056135872020479_U256)
                                .mul_mod(composition_poly[44], PRIME),
                            PRIME,
                        )
                        .add_mod(
                            uint!(427751140904099001132521606468025610873158555767197326325930641757709538586_U256),
                            PRIME,
                        ),
                ),
                PRIME,
            )
            .mul_mod(den_invs[14], PRIME);

            res = res.add_mod(val.mul_mod(alpha_pows[118], PRIME), PRIME);
        }

        {
            // res += val * alpha ** 119.
            let val = input[143].add_mod(
                PRIME.wrapping_sub(
                    uint!(8_U256).mul_mod(composition_poly[43], PRIME)
                        .add_mod(uint!(4_U256).mul_mod(input[139], PRIME), PRIME)
                        .add_mod(uint!(6_U256).mul_mod(composition_poly[44], PRIME), PRIME)
                        .add_mod(input[141], PRIME)
                        .add_mod(input[141], PRIME)
                        .add_mod(
                            uint!(3618502788666131213697322783095070105623107215331596699973092056135872020479_U256)
                                .mul_mod(composition_poly[45], PRIME),
                            PRIME,
                        )
                        .add_mod(input[5], PRIME), // periodic_column/poseidon/poseidon/partial_round_key0
                ),
                PRIME,
            )
            .mul_mod(domains[13], PRIME)
            .mul_mod(den_invs[5], PRIME);

            res = res.add_mod(val.mul_mod(alpha_pows[119], PRIME), PRIME);
        }
        {
            // poseidon/poseidon/partial_round1 (α^120)
            let val = input[178].add_mod(
                PRIME.wrapping_sub(
                    uint!(8_U256).mul_mod(composition_poly[46], PRIME)
                        .add_mod(uint!(4_U256).mul_mod(input[174], PRIME), PRIME)
                        .add_mod(uint!(6_U256).mul_mod(composition_poly[47], PRIME), PRIME)
                        .add_mod(input[176], PRIME)
                        .add_mod(input[176], PRIME)
                        .add_mod(
                            uint!(3618502788666131213697322783095070105623107215331596699973092056135872020479_U256)
                                .mul_mod(composition_poly[48], PRIME),
                            PRIME,
                        )
                        .add_mod(input[6], PRIME),
                ),
                PRIME,
            )
            .mul_mod(domains[14], PRIME)
            .mul_mod(den_invs[7], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[120], PRIME), PRIME);

            // poseidon/poseidon/margin_partial_to_full0 (α^121)
            let val = input[218].add_mod(
                PRIME.wrapping_sub(
                    uint!(16_U256).mul_mod(composition_poly[49], PRIME)
                        .add_mod(uint!(8_U256).mul_mod(input[181], PRIME), PRIME)
                        .add_mod(uint!(16_U256).mul_mod(composition_poly[50], PRIME), PRIME)
                        .add_mod(uint!(6_U256).mul_mod(input[183], PRIME), PRIME)
                        .add_mod(composition_poly[51], PRIME)
                        .add_mod(
                            uint!(560279373700919169769089400651532183647886248799764942664266404650165812023_U256),
                            PRIME,
                        ),
                ),
                PRIME,
            )
            .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[121], PRIME), PRIME);

            // poseidon/poseidon/margin_partial_to_full1 (α^122)
            let val = input[219].add_mod(
                PRIME.wrapping_sub(
                    uint!(4_U256).mul_mod(composition_poly[50], PRIME)
                        .add_mod(input[183], PRIME)
                        .add_mod(input[183], PRIME)
                        .add_mod(composition_poly[51], PRIME)
                        .add_mod(
                            uint!(1401754474293352309994371631695783042590401941592571735921592823982231996415_U256),
                            PRIME,
                        ),
                ),
                PRIME,
            )
            .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[122], PRIME), PRIME);

            // poseidon/poseidon/margin_partial_to_full2 (α^123)
            let val = input[217].add_mod(
                PRIME.wrapping_sub(
                    uint!(8_U256).mul_mod(composition_poly[49], PRIME)
                        .add_mod(uint!(4_U256).mul_mod(input[181], PRIME), PRIME)
                        .add_mod(uint!(6_U256).mul_mod(composition_poly[50], PRIME), PRIME)
                        .add_mod(input[183], PRIME)
                        .add_mod(input[183], PRIME)
                        .add_mod(
                            uint!(3618502788666131213697322783095070105623107215331596699973092056135872020479_U256)
                                .mul_mod(composition_poly[51], PRIME),
                            PRIME,
                        )
                        .add_mod(
                            uint!(1246177936547655338400308396717835700699368047388302793172818304164989556526_U256),
                            PRIME,
                        ),

                ),
                PRIME,
            )
            .mul_mod(den_invs[14], PRIME);
            res = res.add_mod(val.mul_mod(alpha_pows[123], PRIME), PRIME);
        }
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use stylus_sdk::testing::*;
    #[test]
    fn test_compute_result() {
        let mut calldata: Vec<u8> = Vec::new();
        let vm = TestVM::default();
        let mut contract = ConstraintPolyFinalizer::from(&vm);
        for x in INPUT
            .iter()
            .chain(COMPOSITION_POLY.iter())
            .chain(DOMAINS.iter())
        {
            calldata.extend_from_slice(&x.to_be_bytes::<32>());
        }
        for byte in &calldata {
            print!("{:02x}", byte);
        }
        println!();
        let result = contract.compute(&calldata).unwrap();
        let result_word: U256 = U256::from_be_slice(&result);
        assert_eq!(result_word, RESULT);
    }
    use super::*;
    use stylus_sdk::alloy_primitives::{uint, U256};

    const INPUT: [U256; 234] = uint!([
        0x041f59009d6eea6c8d13ea2d4221e632ee2496908d1f4f5c73c1aa2777c925ad_U256, //0
        0x039d6cb187aa47ac255b9bb423fa6811714d6b31059083b7e4b8813ee6d27e83_U256, //1
        0x0758f28f60481b7c23a2b23df777439f207ebe136fa8a11c6358cb9c6293d36b_U256, //2
        0x0767e7579d9fe2f57083878db0e65d8fd17d02d4971d8562c4e894196fcb7364_U256, //3
        0x065f4314fc3dfe1c4f8de071348c1427ef4bec1024c73c2d0d6e3bdff097de9b_U256, //4
        0x050c56d9c9f44b1632b809d70e9179c926f9edd28da62a4c624c8713d79ce395_U256, //5
        0x06acc152add962605f32bfc35939ae9b60f5d5771b606df6886e9b2c1de65ba2_U256, //6
        0x0000000000000000000000000000000000000000000000000000000004000000_U256, //7
        0x0000000000000000000000000000000000000000000000000000000000010000_U256, //8
        0x0000000000000000000000000000000000000000000000000000000000008000_U256, //9
        0x000000000000000000000000000000000000000000000000000000000000031d_U256, //10
        0x0000000000000000000000000000000000000000000000000000000000000001_U256, //11
        0x0000000000000000000000000000000000000000000000000000000000212fd0_U256, //12
        0x0000000000000000000000000000000000000000000000000000000000000005_U256, //13
        0x042d4f5bf719d086cf72bc705d2953f9fcbbf683e27cbf4620b4ad7ebd36b0aa_U256, //14
        0x0048d9f25e6826d2c4927b8c2f38823f7432972cf3a9b1c9a804a6a175106fb5_U256, //15
        0x01dea32fb160f008a7646ca026af012cc61320e00059e8c72e95b2fc7a27674e_U256, //16
        0x0236eaca16d0e3f07d92265f7aa9102a38bdc1d4d9ec085cad8bf8e522c4b232_U256, //17
        0x0000000000000000000000000000000000000000000000000000000000000001_U256, //18
        0x0000000000000000000000000000000000000000000000000000000000000000_U256, //19
        0x000000000000000000000000000000000000000000000000000000000000ffff_U256, //20
        0x009a3a2db35f8bfbde8acb31952fd2ec4bdc906c42fae7f68342254fac2c98af_U256, //21
        0x0000000000000000000000000000000000000000000000000000000000000001_U256, //22
        0x0000000000000000000000000000000000000000000000000000000000000000_U256, //23
        0x0641629c89b59cada06502ecd10bcb848e987d220970d35c6fb2a8eea2238748_U256, //24
        0x0459c42f716f0ef8a7cf623f1dad321589234f0733f5933a2ead4f1fcb20d0e1_U256, //25
        0x07b40c99c2ba755adf7039719d37e0cab9d76e496e8c4b2834f66ce183e62113_U256, //26
        0x049ee3eba8c1600700ee1b87eb599f16716b0b1022947733551fde4050ca6804_U256, //27
        0x03ca0cfe4b3bc6ddf346d49d06ea0ed34e621062c0e056c1d0405d266e10268a_U256, //28
        0x0000000000000000000000000000000000000000000000000000000000214adf_U256, //29
        0x000000000000000000000000000000000000000000000000000000000022cadf_U256, //30
        0x00000000000000000000000000000000000000000000000000000000002acadf_U256, //31
        0x000000000000000000000000000000000000000000000000000000000052cadf_U256, //32
        0x03d8d2c79e51225ca679e36b4795d34603148f22aa2da68432609f1d4586dbc3_U256, //33
        0x04c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad_U256, //34
        0x042d4f5bf719d086cf72bc705d2953f9fcbbf683e27cbf4620b4ad7ebd36b0aa_U256, //35
        0x0048d9f25e6826d2c4927b8c2f38823f7432972cf3a9b1c9a804a6a175106fb5_U256, //36
        0x0236eaca16d0e3f07d92265f7aa9102a38bdc1d4d9ec085cad8bf8e522c4b232_U256, //37
        0x009a3a2db35f8bfbde8acb31952fd2ec4bdc906c42fae7f68342254fac2c98af_U256, //38
        0x0641629c89b59cada06502ecd10bcb848e987d220970d35c6fb2a8eea2238748_U256, //39
        0x0459c42f716f0ef8a7cf623f1dad321589234f0733f5933a2ead4f1fcb20d0e1_U256, //40
        0x057d8f4a8a55ec146a5449bae68c46e072f3966ae58f04418c5f76b9bda0b01c_U256, //41
        0x0093c9c990c9e5a4e488673f3b82850ee9f06e18fbc49da67aec80d3ed4dd705_U256, //42
        0x0082b5e4ab8fa5cf488511e9f61f504e84e01e9e1e70637cc23dad054ffde418_U256, //43
        0x009b5c5498f9ea44cb686f4f186264b33114e09f644cb5cfc2e11d54664c6cd1_U256, //44
        0x01c8225b9880002fa71660ed24fc8af9602eeff23a881a9d6f3a6b69cee6b318_U256, //45
        0x0538c2268edda92db149962226a9ec802c2025458846c98427f38f929960e61b_U256, //46
        0x0294215c3fb0022a5ab4d45c712f083594cc2cd1d93747936595cffcda28f66d_U256, //47
        0x0171049f0db860f62bda2f67e3b119df1d0af7137cf1fa1aaa01beb6a9f4c712_U256, //48
        0x070516c3abf26879bcc9f9a1ac491758715f57cbe63b2c1c05bedfda932d7682_U256, //49
        0x0196cd8fa869af6f924ee99e8695201af1455efbc3c0765f9459c20777998580_U256, //50
        0x06245b11c6f9db8a4748a11e40ac998518d37c00c02f4249ac3bc0aeaff3f24a_U256, //51
        0x00467ce8747dee7fd1fd629aa1d5d06a7d946beec2cd811e670f2b7d849c7368_U256, //52
        0x040015ecd29b31e3a40445f3fee216a9ef1f1f17be6f393a4a8ab4b16a760fa6_U256, //53
        0x07af12d46e53a45d462ea477f763ec86cf5a56360f36590b67bbc3f17ef6075a_U256, //54
        0x0470d301ca7a8a3d77ec1c75b833d83b8e2884af6e1afb9d8f88a4b31f303596_U256, //55
        0x016d859275129d34ce091f9afd3d210eceed129d80215e9dd9d327b18f95e1fc_U256, //56
        0x07a61c13e078eb2b1d17713be8274ca9724b9f65eb5ee76923bbd1a24aa4021e_U256, //57
        0x0639c96448b09d19b1554da6ca5749a01089a0735eb7cf0b120f53d1f9920efc_U256, //58
        0x01e380d81330fa13ea3371f4174759518d69badb87e0fc92830a62507ce89427_U256, //59
        0x04a93aa8d00208af9b4ec8e10c3fd587dc0cadf4ce2a9be9b4d7bd833d12c7e8_U256, //60
        0x0449baab2dd291ab99bc469553c3dadebcf6cba5e68c62ea3067d25ab6f1e89f_U256, //61
        0x079fd66687ede55ed99f35fcda430dbf8cf2a9074965b89e45576529c90eaa4c_U256, //62
        0x06bb841e0385eb69d8b7377031e9f2057ba0f3f19fb8864875231b6611c88980_U256, //63
        0x0606ebc49e35e44fdebc31f43117ac56e5af6e5691ad87c4d6f35140d5e0804d_U256, //64
        0x010f6ec92c1a18a016f3a4d2e7d06a8a9c139751c5e36b3f4d86a3fd5430c23a_U256, //65
        0x00f24396b203890afd33fda3561e83c1add432ead79358443d5c881af52f129b_U256, //66
        0x0136f297ef9c73425685a28ce0721cbf42dc47d423fa29c25ab2e0772b1fbe98_U256, //67
        0x0229dac97f0a5c8d6ba8a885ed50c248676de1c0938708d535b9f6153d70a6d2_U256, //68
        0x0053ac1b279adcdf73bac35cb5411714dd4285e8c9ee1cbb11ab2b36ffb75fb6_U256, //69
        0x04a4505b9cab1fa3b2ebaf9d3790d0cc53ebf6357c7c1c8357cc4af9a761ddfb_U256, //70
        0x00f6bbcb4c64d5d8bb09fb8a82c1b011b0cc40d130719735314348e5bb4a64be_U256, //71
        0x00e747c73e4a67f75c84c5821bc832b0ba8b1f9d9a2ca0f5011223847f3998b1_U256, //72
        0x05d997b7f8eab3b6aa225f55c9f5d6f935894624a645a869816745071be3eedd_U256, //73
        0x02d567c11c0b69781d0b2094dd55a38158325d815d58d467f07f954809606cf6_U256, //74
        0x04078338978a8b344aa879956529e498fdf7cf7eb5a88641a39da3175e72ddd2_U256, //75
        0x00916868cf02526e87f6ba4406f289c5c9e11dba47bc38b3ed88c436b24dd829_U256, //76
        0x0494a45fa699c44a4b8258d4a9424d9a29afb6ebc5c453fa20b24c511b9b0a8b_U256, //77
        0x020d254d18101446316fd6057fdb890b06a50b93424c216861643e91fd05326b_U256, //78
        0x017d371a8f770ed021ff70e8797fd5fb38f2eebd7379a03d29ccf605b94d30b0_U256, //79
        0x07303ad402b182ed4020fcd5cfa54fc86804bc1944ba37fe62ad540926832511_U256, //80
        0x00140cecf4241c622e86088cb49f2c18390d4c093b879399c1216dc5a1cb1900_U256, //81
        0x0176f9274f883dc3d0b2805deee6a334c21898a7d54414d3babc5369af42bfac_U256, //82
        0x0122871531a2746a31f34582cda9646e7aabc231bac1b60bfe3d5dbf723b00a8_U256, //83
        0x07085e952fdacb43ab5dfa5abc7999e28b8320f8c532aff201519885cc276652_U256, //84
        0x01252199656cffddeba73597d1486fce8db840bd53d5c578d203f97e947c0598_U256, //85
        0x01bf3daa0e6a504e624fc9fd91a2c0497bc2391707bd007b6d0cd6f552ba7375_U256, //86
        0x00d906af73e639a6731442bbd27051e0779603660050d6e1e5b4b71ddbf94ed3_U256, //87
        0x0496c34fedd7d85a1ec3fb26448c54a9607528dd0bc0e035e8a2b2ac1e961ed1_U256, //88
        0x016d73bfd11d3f7500bc51b390fe59d8bc7b24006460535ff5c22a08f5a504aa_U256, //89
        0x04f01c9dbcc051babf3dceb55fa29066f8ec1e2976acd00803bbffb1a117d9d7_U256, //90
        0x00aa9518c45d1848e8dc5e689919633f5e53d7cafd9bea0d7d5aaaf1132131f6_U256, //91
        0x001f22dd91869dd65b19ab5cea298fbf4bc8b7eb49b87137e10c46eada0b6e2c_U256, //92
        0x016af8d66759eb86868b4100060753a5f8b7ca90313bf0170667c9ca39c53813_U256, //93
        0x0730596d387f3a0f4c6c35a6a919d5a8eec5b21f8e419f6db50bd503adae72a5_U256, //94
        0x0262cd1106a477ffd91e2623b3a93b9a91e00cba7b882fe90be8fe57e9bd5fb3_U256, //95
        0x055fe091cd0a487c0b4943e64b25e6f8bfa8494ad8232a47945cc9c25b4540f6_U256, //96
        0x008692d95e18c945529347439e406697de2d36be3694164ba24ccd85ce68e3d3_U256, //97
        0x04037dbfa7cf73c7848ebc7b37bbc54a9db84e0e3e5b438fb91ea598fec95ba8_U256, //98
        0x04e643983e693657680ea4454bf8c5dc19dc494b73a9e4011a0c567fd78fcd8a_U256, //99
        0x02f3f9243c26a15fe34e4eceb3c70b1caf37bb8954d3ac65224bc6e059fccd38_U256, //100
        0x05589168392d3f9998bcc9620b438f5a1d44f66465bc1ad6bf937ba4f58e591b_U256, //101
        0x01ffcf455be7a1290f0f57253b31311e978530abb2ed293002a5b30d5399e10b_U256, //102
        0x07edeb009a51e815675f6ed1e9c20715926c33a9faa5b6991db509cbac784aad_U256, //103
        0x04f43459fd2a936b32021f8d264ca726392d5fddb4b2418d9da1e190cc744d3a_U256, //104
        0x065e15d462a7159bfff81a33bbe97d24421259f054db983d3e6e878f736e0914_U256, //105
        0x02bc6236725c0f73890987c40bf98ce0b8c06d4e1a4adc6254c2586b524c1abb_U256, //106
        0x06a5c8a8928ebd0fd462b18e3f3ebb032b10325c6c1b8636f45625f7eecd6b7f_U256, //107
        0x00188520c9829d68b62c3629f1760a9f9438567afaa5b65c56e58a765f87d7e5_U256, //108
        0x0640f51e291baf53db64747c6595629cdcd6bc12bdf2de0aeed9a6b3de7b98ff_U256, //109
        0x052e9745ad5617b125054b54242676b46789413c86925be83c1c15094b9219e6_U256, //110
        0x042733e1d81d6a7d4a541d4f575d52f350487f9e1f6c36ec2c7cb1e23d875ee2_U256, //111
        0x0315a4bfcf6639ce47d76e3e2f480bb89b24c09447bcd025e780b837208112a3_U256, //112
        0x01cdbe9437275fce692b9841575ff5adb432eb3d1ce74a073607c1cbbe4cf7e6_U256, //113
        0x03d791cb22b824d6e998a632b8015165ec2ad50f985f3cf535525a80766e37b8_U256, //114
        0x051bf1141a25c921933873a65165131a2752880897cf25c2def85b679cb43f7d_U256, //115
        0x0512e2adb63ba5d5dd0596a0c463a8eb735d915a6dc829e4405ae170701ea85d_U256, //116
        0x0727f908fe2e13286fc8cb80ebc24ca6e0285775a3b990e121eddf715efc3854_U256, //117
        0x0592f19d6237547fa7f1cdb8076f13fb7897c4e5dc7b6e42677dafe35822c68d_U256, //118
        0x0145ea2a4f7d05afa66db4be4eae729c133b3cd30d457552698cf6b94b3999fc_U256, //119
        0x00b7d6d3841b328265791193552a9de47926922e2fe6ebc0482946b97f92c9d2_U256, //120
        0x00316df6486129d7c2ffaeea0d504e26aba5befbd88f099086611b484d1e9e0c_U256, //121
        0x0225947a33641e816c1e9aa1eae4565a3cb0499374dd6c41031e605794fcec38_U256, //122
        0x01c3d494f75b3368d5990b613e50e0caf10bc7c76a34f56993e68b368473beb6_U256, //123
        0x07b391d396b51a8953de76113e3635269a6647784b510979697f7783336d31f9_U256, //124
        0x0268073e115a29709682103467da12df350f40aa093761ff76f56eb23b0d6d37_U256, //125
        0x074e44019b9584c34e104dfa8e76b55e8f4f4ecb771063f688099a964ca0d29e_U256, //126
        0x07dc9a21788f701dcce3702246d89283ca4d4202d15e150a7e32f6774194f63d_U256, //127
        0x038c4031eb13dbb8a27454eef8fc2ea1b7abc2a1cd3b2421e79e9169af875762_U256, //128
        0x05a17b920fec32de6b3d69f8e61b904a6a9fbb7969ca88b543bd27d7a20ff9e8_U256, //129
        0x07c079d5ab9c70a31ff64e87910a4a906414768d9eb7098e882f2a8c3b739f1b_U256, //130
        0x048b95e6318600e20ec8344acf49819170e5a5f6ceef8ee474ab3cba8ad30563_U256, //131
        0x02b69d7199227616bff1b416260506f2f0e73d83e13e7320620c4fdfbeda165a_U256, //132
        0x028753d0ef231e95edb74079ffbc2f9ca8f4142ab65b79dc93a1d1cf441747e6_U256, //133
        0x02f164c48a3bb8ad46495c7dfef2bdb6dae25c7381832c34a6bb100ba0e16558_U256, //134
        0x064d7bccad5447b8645cc4d24d29851b6082c91e07776fcf894e2e7aa85b89df_U256, //135
        0x025cdc88866af0b383d5348d7c24acd7574c6727f351d2a19624b7e54dfc1b68_U256, //136
        0x00a849b8d874616821daaa860d25cc462b63dfea13ee463d1cfc414f23d96427_U256, //137
        0x02f6fcf013f0236bef2214bb45def53b5f517a1eca64375ba1d99b28bd2df9ea_U256, //138
        0x01e16c35dde2cd3b868e5bca632c67b46c9808b4e7f18d9aee431d0299769582_U256, //139
        0x00a771479a9a2092c36a0891d254622794be7463dbe25cbed3e6b6412df69b2b_U256, //140
        0x0530bf5784132e27fdd921cf4a12eaea7e66dc963514d4f2ea64d5139589f2dd_U256, //141
        0x03449ca7d2c6c0498e47dba4f70a9a5a51220095457296172c5647c6b37c5119_U256, //142
        0x010b5930b33099b439300db3fa2cc1215cc05ab641a95e17f5329fca89bbbb4c_U256, //143
        0x051660cf6a28773f54d170870901ab95a15934ca31817b86df0fd3aa9039c599_U256, //144
        0x03062f0b60af6bee3b2e3c7094958297306d635dadd6cf7b20a9099a05aacb75_U256, //145
        0x03c9eb851066f3cf2ef8cb3cf759e7144b72306fa064cbd4b9d3770f8653daf7_U256, //146
        0x06a5dbeffa902cf75f3630a67f55c6ac3fc3018cef52af10143d327280b0604d_U256, //147
        0x054189f42ae47a878adf2901632d6890ef83057e367d91d9f83d2e8775b5daf3_U256, //148
        0x074c43be295467d331b01fb9b2506c572eb687e4f619586f08557de0bbea073c_U256, //149
        0x049abfd07a91062ff8183c009d529cec7afc50a2118f81cb039e3672b70fbad8_U256, //150
        0x0633b1f541b3858bfd74a080e6f0f15b9ddf4d6c2ff03d674f38771ce2095dc8_U256, //151
        0x03e7464aa9e7fc9783440a1876bbcda69e6b234b5bf415550d30f36011c9e2d0_U256, //152
        0x0707d50ecb6661446fbe0d5a14e468e501f05df2f8e695cdcbcd7338f7850084_U256, //153
        0x035a4f4540b41dce432ec725e7cde228afa3f1fdebdf0a6421d5934e66e262af_U256, //154
        0x0425673bf635b5414ca65fcda31f1438b1cc412ed87162fd32bce1e2fc8dbd26_U256, //155
        0x04ad7ff2e48d506861fa02abf163487b4bdb06299fe3c9368defa090327c5d2f_U256, //156
        0x00b7ad96549f364798ca66b0b7b8adfe503fc9ad7d8c3a4e1b0913ae9cf29e4c_U256, //157
        0x01e8ec07f4ef259531d94652ae9c0b79d3ac637c663960e6098ab722c51379b1_U256, //158
        0x02dae8614cfdc57df8080c41ae2062fa05fa471510e987b4880f96e9d047d64b_U256, //159
        0x011908b37a7aadc55680efc1d7744cad713f749178f8ddf8f45be17835aad680_U256, //160
        0x06d0ddde10069771e39162c1819b92a813fe17684c08902771db0863f71d9495_U256, //161
        0x07e6c3d355a7a55a7b56b330d710bd4dd2bb882b0a623024506b052a59c45612_U256, //162
        0x073c69c1f53b0d01db1bd6378149d9ff2994eab71dd8280de9d517b69a5e085b_U256, //163
        0x064a0667cfda89af297b071a940421aa71316fcfebd54e06aa3d17bccb424cb3_U256, //164
        0x03ca480cfd4d4884bf7b29e9608616fb5ffcf610af9254be9e38401641e8903b_U256, //165
        0x07e1c8e9adf7efceb0a2fcae81d8538ea5bcb398061696c6f0be3df955e27900_U256, //166
        0x0034f752a2ec7e8c72a8ae11c126b1d9665e0bfdb046a3b15ed8b339bc98fe75_U256, //167
        0x0692a1e07452637bd9bbbe80abbbe7d3cd73dbc8cce5e52c9b0afbe9f1064b99_U256, //168
        0x042ce7adaa87f0de4203c5ec68ae22fe7476b2c52e424681d9500a81581a22b3_U256, //169
        0x043c7c79cc1529e1c4a05c6e633d5bc9d724b3777d5485d04d5727994f18e769_U256, //170
        0x01c80d11065d969445b5b41d5d29b56e327d5b06935df5471dce7f915278afad_U256, //171
        0x0686b975ca5524470704e12216efaee921e8aab7fbe2b2241ae5244c49108eb4_U256, //172
        0x02f7e92dc54ca85b947263d4ed15f4c079be9ab51cc64563d6bec5de596af8a9_U256, //173
        0x01025d3cb9a4f129bc45a122df45de89f5a4b5404a01f0bc6dda9fc47415ccaf_U256, //174
        0x037cbf73be4770881113f44d300959c55a9e8a4102c357122c491153fe7846c8_U256, //175
        0x065f783888d49446953dd71c0daf474875a4366ffc01292900d6e7f60187633b_U256, //176
        0x079f9d82960921ead6147f4a2d46372eb1d2813428952c9dfefc9d871559913f_U256, //177
        0x02435c0c48ae1600d0a47983ef44b952184c1a208d3fedc6c32e07877947da20_U256, //178
        0x01d6200a2490287326cde0feea3cdb2eeb44a9ce5b002240397047a30191856a_U256, //179
        0x01ae1549b835654108d17e9872af976df75dc6a613a1182c4a0e623760ef5f1a_U256, //180
        0x072dc21b47ba765b1a56801af024cd5c96de3ce3c2c2003c43ee8bc9e6213835_U256, //181
        0x0029375037e73adffaea91fb1b2cb2e3c5ee06e0dc6958755a96301ba6f1dd8e_U256, //182
        0x05cc58166f4a1c8bb707e86fe89e1bd9ae609d002d77173b5c886791a9109cd2_U256, //183
        0x048d63a47e22ba2b66c532bf645271d2eb9b0c0b08ba134af5f7d8cbfcbd7c63_U256, //184
        0x0268ed2d8dc8435948c8866989fc86fca76902f865cfad41d4588a1364af7979_U256, //185
        0x0726589d25d799bdbb045e9614c43a82dee6301c06b21179352290ff22670329_U256, //186
        0x0421e123410b7fd4933499620192d62b2f07ea71392d023d3a489f0784e2caa1_U256, //187
        0x0689769b669833a0cc12ffaebef1a70b93b1ea4da3cae3038202ce6374f35ae9_U256, //188
        0x0298fb8c52f6cb6787ab9b557ed4e158276265beeb76771a59248a5db160f74d_U256, //189
        0x0658da49be0a8fff97211f0e66d081d372a3a644ba5c0c50628b4eccba57d003_U256, //190
        0x0153a4d7d12a2c3f5c88abd3d6d8f402f07b1db91b9206213c7d9f6bc6b34461_U256, //191
        0x05c8b5d965a30055ed02667eaa2568fbe5cbb2c204acd1df5c186e9cfed72b9e_U256, //192
        0x0301ca1f7b2b3a95d8f8f2804e34768f351af21690fc27b4d3afc738afb72b44_U256, //193
        0x078a2e76967833c742bdd0716e5b8ea7a35b8d3784cdbd1f88a1179f30b7a689_U256, //194
        0x00536e95ee5616aabbbc19be8889c5b065629c7463be346d5bb8162f6fffbdde_U256, //195
        0x05b14d2301f81625a327d150c7884449a4809cb91d490f36b4a521201a337c09_U256, //196
        0x048bc4d060922e4ec7669dcbe795b69b0d663f539b2f62cae0cf500108edfd4c_U256, //197
        0x04a3da4a21771ebca51f9b166aca11b2e72026aa2760e8d4e064e587a8c577c9_U256, //198
        0x0576102d9092a726c51b785785cf7b9b264ec87c577817e3f033ae6df60033a2_U256, //199
        0x01c9ee2ae1719c1652c5f78c7e9a839f2e205a1df4c119e5413c002840744c7e_U256, //200
        0x00fa7050de0fd2831519282c3321de6ada2c92cd8767d71e0a130e5db0710d26_U256, //201
        0x033b2e1ff29189ed6298a207f4e1c2d5154c2f297dee6847635204f4303d7299_U256, //202
        0x01ad15781452c192018fea663fbe5478bece1723b6ab4b7bc17c7feb46af1345_U256, //203
        0x04e602004d395cb051e1b20cc201077b25c1b4307308df18297b4ba4fe2617d9_U256, //204
        0x0467d0ecaad754431de331c31dc8c1b70aa898e6cc0cf6873b5b7b2abb85548f_U256, //205
        0x04f89c23de4bed08ece304fb19aea4879dba1c762c34a3a4740995ca1f38ac91_U256, //206
        0x01ad482e7b79bb31fba62a8097d118479e4a09ac49445912a8139c467ec79826_U256, //207
        0x04b3c11552648bc5cb63ee017c500d48cfe376ad681b012ddb393d6118dc4974_U256, //208
        0x00c5caad63688f9910d5fa508f1bec5ff0bfc33237bdd4c9780976332ea4aa32_U256, //209
        0x05070e3e35e081dd243c83594333829d4686d17d7c0c7fbac4065af74e3b8522_U256, //210
        0x013dd86c6101cf137216cc4825adb91eb11b0d80fc5e5db1ee6960c629aec5d1_U256, //211
        0x02417a02acc7157a7dae3ab35201ccbb2f1ded439e66f57386ff912a65b73716_U256, //212
        0x02c21e63b8b85e76601feb088f3583045e42617e814bc6c5f2597f1677fbed18_U256, //213
        0x04aa5e66eb7dbb0e0d8537a31bed0f7fd3f4f63d5b543b3f3dede90b440c8828_U256, //214
        0x048762a7ae3ff44146e7bcd5f0d8b73bed4bc83b273288e9ecb7d16ef806cfc0_U256, //215
        0x0214ad4a7d3624f723df4f2470199c1a9c5fe403b30eeca59df8dbbe3867d70c_U256, //216
        0x010b44851564a01088d10e397018136de302da854650243f13bacddee70d953d_U256, //217
        0x001d5cf117c59526493f8fa4a3ebf0393f79e2ffb03dbaf38ea69f5e27a229bc_U256, //218
        0x019bc10ac171104884ce6ac388e964af8e0c76ebd6f3ee943e6d1771e488d0e0_U256, //219
        0x051b0dc486c37b515810a33a49839069447c2143eba8040b27a5b86947acdaac_U256, //220
        0x06ce0a62d90406bfb83b51aa5ad31ea0cc4b68cf2435441b52e2e30b6de87c9d_U256, //221
        0x021dc43f3a2925bf27ca76821d1e193a90b5acde0e9ec0357e038cd04f26e687_U256, //222
        0x031cc4f2a30d4206cdc4c0c39c253252aa91342bb8c98588ec4f1df66794483e_U256, //223
        0x007816ee172d21823ee1a7d3c76ecfa25bb442e3988a0f713f7f4fb16036307a_U256, //224
        0x01e4365b0b80ff8cdb775717cbb9def50712f9e011ac832b1f2ea1444eb3528c_U256, //225
        0x06357ee7c3a81ff69f64aa8f60872f229809488da16ae04da6bc705eebc496ee_U256, //226
        0x022540cee923d15ff3dba817db508029899ed6f6df70b2ba944a2b2afd7db0c7_U256, //227
        0x01e84306f8186fa607e44fdb603a248a564e2b411a452f32538f8942b55ed131_U256, //228
        0x021d4eba06bc6a62e07df091a35eeb0d505e2a4fa31a75c857a0cd5a3dd17943_U256, //229
        0x06cdacc081edbf793a56de134b1da9da2b08f009044685eb6e86acfb9eb1a09a_U256, //230
        0x03c7e587c4d29c93308c1bdea48cc8e5fd76451b92d1c9a9ce1ee32cda19e29d_U256, //231
        0x03198f652bcc9e77c3e0bca8281560fac62d04f23e1a185e2536bf260987f75e_U256, //232
        0x008102625fef272e42af824f71b1d7b68de954eb5bcbb5befd1ce1b0f85afcde_U256, //233
    ]);

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

    const COMPOSITION_POLY: [U256; 52] = uint!([
        0x078e5e0039aa9a17537e436b4f43e471e03030dcbee3d6acf67126c94d520ed6_U256, // 0x1d40 (234)
        0x050b179d67f9e9f67d3bad74ce694ec070b700baef3c8094e46c4680c87f06a2_U256,
        0x00107f6e0f7da4d8fbdfed69444bdc150287cba1d5d83a5d5cc7ef98e50ef941_U256,
        0x07569e0e7ac4adf6448334a8d7a8b1f907eea56729fa87951f534c449c24e6e4_U256,
        0x038dcae60dc3c35c4261307915a2233184d28e3c10f0bd789f787da1b64d193c_U256,
        0x07b2181e243f404f0300758ca9ccd4775ab63eaadf53535e1192528f863f684a_U256,
        0x0366d717b5d39024b2463c248b1eeb2e3a4c477bb07ba1e29e83ff018399da10_U256,
        0x05976140ddfdfe8aa34ddbe8fd00f8b01daaa4233a94400cde1d69b3a6bb0b7a_U256, // 0x1e20 (241)
        0x074faf8947ef3134a76b7265ce1347aa4d52d5b6359ccab271cc44bb4f6bb230_U256, // 0x1e40 (242)
        0x03d77ba45b1f099a982c26649f1ed7228ed499d45eba3f5cdd0b5bcba3fa6b82_U256, // 0x1e60 (243)
        0x054e176c1a75f87d03bda762053bed10bf9e66fa4361f1cc3be240aa17b1a0ee_U256, //244
        0x01430baeac6cff7fc0c856505ea4431c93e25b0e234f8eca08f4f9d69d98e819_U256, //245
        0x06cd6cd0d95e8ff356566b8c86fc3c0fb3094cd7330061d048aa7a8b40959c2f_U256, //246
        0x0195c7dce0554fd3dbd9dd3fbdb9961df04e5f746dd83e61dbe255500004719e_U256, //247
        0x079ccb52464c205acdcfb733bb4a2dd25ca853b45f275fcddb733024bf65f236_U256, //248
        0x074bfd3b799bd156b1b4334bc55a86e822b65d5f55d6f7dd3c7b725c83650a77_U256, //249
        0x05b5acb62c57023f66180bdd6782b1ffcf0ad885ecd86aa261c6f171dba03899_U256, // 0x1f40 (250)
        0x0046510ecf478ac989f4d6b2a411a3169f562dbf45ef0ea9d1f9c21aafb0541d_U256, //251
        0x04a1f043f5f3e94b17a6fd04101a3d9c506a72aba00287237b132cce6c8a00f4_U256, //252
        0x02214d6ab420c70093da3d232cee87bbea55d3d1a9638fcb925b846cfa4dddc2_U256, //253
        0x03c627fbbe31292276a584584d6d557eb78eb4f3511bf5f2f5ac5cab644441f9_U256, //254
        0x07bb9150a211f9823e0deda06293fc8dd339d60e02cd3d5ec377f5583b9af949_U256, //255
        0x063d15521feea038191efe428e82397d80f97d5af4b5bbba2bd27ec4a5443162_U256, //0x2000 (256)
        0x01c2eaade0115fd8e6e101bd717dc6827f0682a50b4a4445d42d813b5abbcea0_U256, // 257
        0x04ad7ff2e48d506861fa02abf163487b4bdb06299fe3c9368defa090327c5d2f_U256, //258
        0x00aa9223a4fda7919b7658140033f9d95669699146c2c83dbb99462afa21089c_U256, //0x2060 (259)
        0x040c91059c7f569989ed4686a87561e33d3daa3f2e771c7f4fb5b143cdaf645f_U256, //260
        0x03e084e0a38eb4b93e8eb4b71003a037b039758c2d68d76a3953649934a654b9_U256, //261
        0x05f9ac422f2baf440b37ffc577abfce6e6cba1fa5063174c58f5161e8a635a70_U256, //262
        0x03130d09bf3df013e356da6d7e827973b5f867cb6354e91c87f992c7518cd55f_U256, //263
        0x04f0831145b5002f55c431b3508473463a86eb7ff37eb81de33256b72f22f3b0_U256, //264
        0x024daf76f55fe3a50ccf26bbf49014861514de35d5f60b40408c46d98e0d6a4a_U256, // 0x2120 (265)
        0x03c28524c8341c8341cd6ee19b3ad318681c01e398317f4f94f2d1618928bf82_U256, // 0x2140 (266)
        0x06d931d4d0304de8b058c8f93ff57b7ef7752d4c25797c47efbf6a49b503a5c7_U256, //267
        0x02c68633147ddc68c0671b54ae4d5a668626822669dc356970c4ff7115a4aeb5_U256, //268
        0x03e76113d7f454813a3ca2591c1b6978058165fd3effb3adffdde7a0b183ae50_U256, //269
        0x021b65170112519dd7bd3b9f9e9c0e7c7a9fc358387748685b758792b8558a5a_U256, //270
        0x050df35421c033a0a92755521362408a5f3a88ec732eaff5467c8256e9c4d139_U256, //271
        0x05fc0a9dc7590923005d9f066ce04a424005bb392f1d4af8fb46e2f195298b25_U256, //272
        0x00e21fe3841ce1f5f40379894f4a573967c06541c22444acad7c106ea35b2ef3_U256, //273
        0x04b2a475efb54e266b136de9041b93d00a6169c6fd1955db803e9129ae1baedb_U256, //274
        0x0321b29ddfe054dcb3bc42d3bf1e69e947c25e85092dcc7b1762bd8e31f0f84d_U256, //275
        0x0602b0310da1702ad0a3ad4a09c83e7a69654b27ada35149b12ce3ecd64d3dff_U256, //276
        0x02d8c796d7f5ce296325a2cff214954a94f5ff1613912b0f7931fb8daae592a4_U256, //277
        0x01d36478e0d90e980370c23e47a12d47f21a9540ff633583506b2b4698351189_U256, //278
        0x04973ab88d3948766a4a137a67475638759974e1aac537c11e275b5b970170a6_U256, //279
        0x034048560f6a931ca89ce0a7f739ca815713ba39fbed9b5bebdea866bf48533d_U256, //280
        0x0493c036f3e3bb7b17ae82dd373e0f5057cdd10513c4592cf19ab272608ceb47_U256, //281
        0x00f720a04f2b224b5fc2ed896ab229d5a10c0a6e8cd56c9e9064acb24181dc54_U256, //282
        0x00923750efedd307797f6a537e2404adb5d2937fd9619bac9bb20ad0628745bc_U256, //283
        0x062b67bdc72e40797acd4d574bcc745e02a5052c2434b4507d5cc947dcfae72a_U256, //284
        0x04b890835f6e76126680bd97b8c8d6305750aa8076a955238388c0c88badc3f1_U256, // 0x23a0 (285)
    ]);

    // 011c9786266bae42dde1f8aa500daa5d15789f42f645109651766156e8846ce0
    const RESULT: U256 =
        uint!(0x06830dfba344bbbb4521412ab453a5883b76d7649286a365017d2eb2984ad636_U256);
}