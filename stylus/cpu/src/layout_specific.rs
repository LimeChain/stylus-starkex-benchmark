// Lyubo: We should use that as a contract that is inherited by the cpuverifier
pub mod layout_specific {
    use crate::require;
    use crate::stark_verifier::VerifierError;
    use stylus_sdk::alloy_primitives::U256;

    pub fn get_layout_info(&self) -> Result<(U256, U256), Vec<u8>> {
        let public_memory_offset: U256 =
            U256::from(public_input_offsets::OFFSET_N_PUBLIC_MEMORY_PAGES);
        let selected_builtins: U256 = U256::from(
            (1 << OUTPUT_BUILTIN_BIT)
                | (1 << PEDERSEN_BUILTIN_BIT)
                | (1 << RANGE_CHECK_BUILTIN_BIT)
                | (1 << ECDSA_BUILTIN_BIT)
                | (1 << BITWISE_BUILTIN_BIT)
                | (1 << EC_OP_BUILTIN_BIT),
        );
        Ok((public_memory_offset, selected_builtins))
    }

    pub fn safe_div(
        numerator: U256,
        denominator: U256,
        err_msg: &str,
    ) -> Result<U256, VerifierError> {
        require!(
            denominator != U256::ZERO,
            "The denominator must not be zero"
        );
        require!(
            numerator % denominator == U256::ZERO,
            "The numerator is not divisible by the denominator."
        );
        Ok(numerator / denominator)
    }

    fn validate_builtin_pointers(
        initial_address: U256,
        stop_address: U256,
        builtin_ratio: U256,
        cells_per_instance: U256,
        n_steps: U256,
        builtin_name: &str,
    ) -> Result<(), VerifierError> {
        Ok(())
    }

    fn prepare_for_oods_check(ctx: &[U256]) {
        let oods_point = ctx[MM_OODS_POINT];
        let n_steps = 1 << ctx[MM_LOG_N_STEPS];
        // Lyubo: define PEDERSEN_BUILTIN_RATIO and PEDERSEN_BUILTIN_REPETITIONS in consts.rs
        let n_pedersen_hash_copies = safe_div(n_steps, PEDERSEN_BUILTIN_RATIO * PEDERSEN_BUILTIN_REPETITIONS);
        // Lyubo: Use PrimeFieldElement0::fpow
        let z_point_pow_pedersen = fpow(oods_point, n_pedersen_hash_copies);
        // Lyubo: Define pedersen_points_x and pedersen_points_y in the storage
        ctx[MM_PERIODIC_COLUMN__PEDERSEN__POINTS__X] = pedersen_points_x.compute(z_point_pow_pedersen);
        ctx[MM_PERIODIC_COLUMN__PEDERSEN__POINTS__Y] = pedersen_points_y.compute(z_point_pow_pedersen);

        ctx[MM_DILUTED_CHECK__PERMUTATION__INTERACTION_ELM] = ctx[MM_INTERACTION_ELEMENTS + 3];
        ctx[MM_DILUTED_CHECK__INTERACTION_Z] = ctx[MM_INTERACTION_ELEMENTS + 4];
        ctx[MM_DILUTED_CHECK__INTERACTION_ALPHA] = ctx[MM_INTERACTION_ELEMENTS + 5];

        ctx[MM_DILUTED_CHECK__FINAL_CUM_VAL] = compute_diluted_cumulative_value(ctx);

        let n_poseidon_hash_copies = safe_div(1 << ctx[MM_LOG_N_STEPS], POSEIDON__RATIO);
        let z_point_pow_poseidon = fpow(oods_point, n_poseidon_hash_copies);

        // Lyubo: Define poseidon_poseidon_full_round_key0, poseidon_poseidon_full_round_key1, poseidon_poseidon_full_round_key2, poseidon_poseidon_partial_round_key0, poseidon_poseidon_partial_round_key1 in the storage
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY0] = poseidon_poseidon_full_round_key0.compute(z_point_pow_poseidon);
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY1] = poseidon_poseidon_full_round_key1.compute(z_point_pow_poseidon);
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY2] = poseidon_poseidon_full_round_key2.compute(z_point_pow_poseidon);
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__PARTIAL_ROUND_KEY0] = poseidon_poseidon_partial_round_key0.compute(z_point_pow_poseidon);
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__PARTIAL_ROUND_KEY1] = poseidon_poseidon_partial_round_key1.compute(z_point_pow_poseidon);
    }

    fn compute_diluted_cumulative_value(ctx: &[U256]) -> U256 {
        let z = ctx[MM_DILUTED_CHECK__INTERACTION_Z];
        let alpha = ctx[MM_DILUTED_CHECK__INTERACTION_ALPHA];
        let diff_multiplier = 1 << DILUTED_SPACING;
        let diff_x = diff_multiplier - 2;
        let mut p = 1 + z;
        let mut q = 1;
        let mut x = 1;

        for i in 1..DILUTED_N_BITS {
            x = x.add_mod(diff_x, K_MODULUS);
            diff_x = diff_x.mul_mod(diff_multiplier, K_MODULUS);
            let x_p = x.mul_mod(p, K_MODULUS);
            let y = p.add(z.mul_mod(x_p, K_MODULUS));
            q = (q.mul_mod(y, K_MODULUS).add(x.mul_mod(x_p, K_MODULUS))).add_mod(q, K_MODULUS);
            p = p.mul_mod(y, K_MODULUS);
        }
        let res = p.add_mod(q.mul_mod(alpha, K_MODULUS), K_MODULUS);
        res
    }
}
