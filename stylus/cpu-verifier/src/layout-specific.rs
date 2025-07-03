#![cfg_attr(not(any(test)), no_main)]
extern crate alloc;
use alloc::vec::Vec;

// use macros::require;
#[path = "prime-field-element0.rs"]
mod prime_field_element0;

use prime_field_element0::PrimeFieldElement0;
use stylus_sdk::alloy_primitives::{U256, uint};

pub struct LayoutSpecific {}

impl LayoutSpecific {
    pub const OUTPUT_BUILTIN_BIT: usize = 0;
    pub const PEDERSEN_BUILTIN_BIT: usize = 1;
    pub const RANGE_CHECK_BUILTIN_BIT: usize = 2;
    pub const ECDSA_BUILTIN_BIT: usize = 3;
    pub const BITWISE_BUILTIN_BIT: usize = 4;
    pub const EC_OP_BUILTIN_BIT: usize = 5;
    pub const KECCAK_BUILTIN_BIT: usize = 6;
    pub const POSEIDON_BUILTIN_BIT: usize = 7;

    pub fn get_layout_info(&self) -> Result<(U256, U256), Vec<u8>> {
        let public_memory_offset = U256::from(21);
        let selected_builtins = U256::from(
            (1 << LayoutSpecific::OUTPUT_BUILTIN_BIT)
                | (1 << LayoutSpecific::PEDERSEN_BUILTIN_BIT)
                | (1 << LayoutSpecific::RANGE_CHECK_BUILTIN_BIT)
                | (1 << LayoutSpecific::ECDSA_BUILTIN_BIT)
                | (1 << LayoutSpecific::BITWISE_BUILTIN_BIT)
                | (1 << LayoutSpecific::EC_OP_BUILTIN_BIT),
        );
        Ok((public_memory_offset, selected_builtins))
    }

    pub fn layout_specific_init(ctx: &mut [U256], public_input: &[U256]) {
        let output_begin_addr = public_input[9];
        let output_stop_ptr = public_input[10];
        // require!(output_begin_addr <= output_stop_ptr, "output begin_addr must be <= stop_ptr");
        // require!(output_stop_ptr < U256::from(18446744073709551616), "Out of range output stop_ptr.");

        let n_steps = U256::from(1) << ctx[1274];
        ctx[346] = public_input[11];
        LayoutSpecific::validate_builtin_pointers(ctx[346], public_input[12], U256::from(128), U256::from(3), n_steps);

        ctx[344] = uint!(2089986280348253421170679821480865132823066470938446095505822317253594081284_U256);
        ctx[345] = uint!(1713931329540660377023406109199410414810705867260802078187082345529207694986_U256);
        ctx[347] = public_input[13];
        LayoutSpecific::validate_builtin_pointers(ctx[347], public_input[14], U256::from(8), U256::from(1), n_steps);

        ctx[335] = U256::from(1);
        ctx[348] = public_input[15];
        LayoutSpecific::validate_builtin_pointers(ctx[348], public_input[16], U256::from(8), U256::from(5), n_steps);

        ctx[339] = U256::from(1);
        ctx[340] = U256::from(0);

        ctx[349] = public_input[17];
        LayoutSpecific::validate_builtin_pointers(ctx[349], public_input[18], U256::from(8), U256::from(6), n_steps);
    }


    pub fn validate_builtin_pointers(
        initial_address: U256,
        stop_address: U256,
        builtin_ratio: U256,
        cells_per_instance: U256,
        n_steps: U256
    ) {
        // require!(
        //     initial_address < U256::from(18446744073709551616),
        //     "Out of range begin_addr."
        // );
        let max_stop_ptr = initial_address + cells_per_instance * LayoutSpecific::safe_div(n_steps, builtin_ratio);
        // require!(
        //     initial_address <= stop_address && stop_address <= max_stop_ptr,
        //     "Invalid stop_ptr."
        // );
    }

    pub fn safe_div(
        numerator: U256,
        denominator: U256
    ) -> U256 {
        // require!(
        //     denominator != U256::ZERO,
        //     "The denominator must not be zero"
        // );
        // require!(
        //     numerator % denominator == U256::ZERO,
        //     "The numerator is not divisible by the denominator."
        // );
        numerator / denominator
    }

    pub fn prepare_for_oods_check(ctx: &mut [U256]) {
        let oods_point = ctx[351];
        let n_steps = U256::from(1) << ctx[1274];
        let n_pedersen_hash_copies = LayoutSpecific::safe_div(n_steps, U256::from(128));
        
        let z_point_pow_pedersen = PrimeFieldElement0::fpow(oods_point, n_pedersen_hash_copies);
        // Lyubo: Define pedersen_points_x and pedersen_points_y in the storage
        // ctx[317] = pedersen_points_x.compute(z_point_pow_pedersen);
        // ctx[318] = pedersen_points_y.compute(z_point_pow_pedersen);

        ctx[338] = ctx[355];
        ctx[341] = ctx[356];
        ctx[342] = ctx[357];
        ctx[343] = LayoutSpecific::compute_diluted_cumulative_value(ctx);
        
        let n_poseidon_hash_copies = LayoutSpecific::safe_div(U256::from(1) << ctx[1274], U256::from(8));
        let z_point_pow_poseidon = PrimeFieldElement0::fpow(oods_point, n_poseidon_hash_copies);

        // Lyubo: Define poseidon_poseidon_full_round_key0, poseidon_poseidon_full_round_key1, poseidon_poseidon_full_round_key2, poseidon_poseidon_partial_round_key0, poseidon_poseidon_partial_round_key1 in the storage
        // ctx[319] = poseidon_poseidon_full_round_key0.compute(z_point_pow_poseidon);
        // ctx[320] = poseidon_poseidon_full_round_key1.compute(z_point_pow_poseidon);
        // ctx[321] = poseidon_poseidon_full_round_key2.compute(z_point_pow_poseidon);
        // ctx[322] = poseidon_poseidon_partial_round_key0.compute(z_point_pow_poseidon);
        // ctx[333] = poseidon_poseidon_partial_round_key1.compute(z_point_pow_poseidon);
    }

    fn compute_diluted_cumulative_value(ctx: &[U256]) -> U256 {
        let z = ctx[341];
        let alpha = ctx[342];
        let diff_multiplier = U256::from(16);
        let mut diff_x = U256::from(diff_multiplier) - U256::from(2);
        let mut p = U256::from(1) + z;
        let mut q = U256::from(1);
        let mut x = U256::from(1);

        for i in 1..16 {
            x = x.add_mod(diff_x, PrimeFieldElement0::K_MODULUS);
            diff_x = diff_x.mul_mod(diff_multiplier, PrimeFieldElement0::K_MODULUS);
            let x_p = x.mul_mod(p, PrimeFieldElement0::K_MODULUS);
            let y = p + z.mul_mod(x_p, PrimeFieldElement0::K_MODULUS);
            q = (q.mul_mod(y, PrimeFieldElement0::K_MODULUS) + x.mul_mod(x_p, PrimeFieldElement0::K_MODULUS)).add_mod(q, PrimeFieldElement0::K_MODULUS);
            p = p.mul_mod(y, PrimeFieldElement0::K_MODULUS);
        }
        let res = p.add_mod(q.mul_mod(alpha, PrimeFieldElement0::K_MODULUS), PrimeFieldElement0::K_MODULUS);
        res
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    mod test_constants;
    use stylus_sdk::{
        alloy_primitives::{U256, uint},
    };

    // #[motsu::test]
    // fn test_compute_diluted_cumulative_value() {
    //     let mut ctx = test_constants::get_ctx_compute_diluted_cumulative_value();
    //     let diluted_cumulative_value = LayoutSpecific::compute_diluted_cumulative_value(&ctx);
    //     assert_eq!(diluted_cumulative_value, uint!(1552215061468209516830163195514878071221879601444981698864155012436627340325_U256));
    // }

    // #[motsu::test]
    // fn test_layout_specific_init() {
    //     let mut ctx = test_constants::get_ctx_layout_specific_init();
    //     let public_input = test_constants::get_public_input();
    //     LayoutSpecific::layout_specific_init(&mut ctx, &public_input);
    //     assert_eq!(ctx[346], uint!(2392152_U256));
    //     assert_eq!(ctx[344], uint!(2089986280348253421170679821480865132823066470938446095505822317253594081284_U256));
    //     assert_eq!(ctx[345], uint!(1713931329540660377023406109199410414810705867260802078187082345529207694986_U256));
    //     assert_eq!(ctx[347], uint!(2490456_U256));
    //     assert_eq!(ctx[335], uint!(1_U256));
    //     assert_eq!(ctx[348], uint!(3014744_U256));
    //     assert_eq!(ctx[339], uint!(1_U256));
    //     assert_eq!(ctx[340], uint!(0_U256));
    //     assert_eq!(ctx[349], uint!(5636184_U256));
    // }

}
