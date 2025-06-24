use crate::consts::{input_offsets::*, memory_offsets::*, stark_parameters::*};
use crate::require;
use stylus_sdk::alloy_primitives::{uint, U256};
use stylus_sdk::alloy_sol_types::sol;
use stylus_sdk::prelude::SolidityError;

use crate::public_memory_offsets::{offset_page_size, public_input_length};

pub struct Fri {}

// Lyubo: Add merkle_statement_contract into the storage of the contract
impl Fri {

    fn fri_verify_layers(ctx: &[U256], proof: &[U256]) {
        let fri_ctx = MM_FRI_CTX;
        require!(MAX_SUPPORTED_FRI_STEP_SIZE == FRI_MAX_STEP_SIZE, b"MAX_STEP_SIZE is inconsistent in MemoryMap.sol and FriLayer.sol");
        init_fri_groups(fri_ctx);

        // Lyubo: Use get_channel_ptr from stark_verifier.rs
        let channel_ptr = get_channel_ptr(ctx);
        let merkle_queue_ptr = get_merkle_queue_ptr(ctx);

        let fri_step = 1;
        let n_live_queries = ctx[MM_N_UNIQUE_QUERIES];

        ctx[MM_FRI_QUERIES_DELIMITER] = 0;

        for i in 0..n_live_queries {
            // Lyubo: Use fmul from PrimeFieldElement0::fmul
            ctx[MM_FRI_QUEUE + FRI_QUEUE_SLOT_SIZE * i + 1] = fmul(
                ctx[MM_FRI_QUEUE + FRI_QUEUE_SLOT_SIZE * i + 1],
                K_MONTGOMERY_R
            );
        }

        let fri_queue_ptr = MM_FRI_QUEUE;
        let fri_step_sizes = get_fri_step_sizes(ctx);
        let n_fri_steps = fri_step_sizes.len();

        while fri_step < n_fri_steps {
            let fri_coset_size = 2**fri_step_sizes[fri_step];
            let n_live_queries = compute_next_layer(ctx, proof, channel_ptr, fri_queue_ptr, merkle_queue_ptr, n_live_queries, fri_ctx, ctx[MM_FRI_EVAL_POINTS + fri_step], fri_coset_size);
            // Lyubo: Use verify_merkle from merkle_statement_verifier.rs
            // Lyubo: Move u256_to_bytes to utils.rs(stark_verifier.rs)
            verify_merkle(ctx, &merkle_queue_ptr, &u256_to_bytes(ctx[MM_FRI_COMMITMENTS + fri_step - 1]), &n_live_queries);

            fri_step += 1;
        }

        verify_last_layer(ctx, n_live_queries);
    }

    fn init_fri_groups(ctx: &[U256], fri_ctx: &[U256]) {
        let fri_group_ptr = fri_ctx + FRI_CTX_TO_FRI_GROUP_OFFSET;
        let fri_half_inv_group_ptr = fri_ctx + FRI_CTX_TO_FRI_HALF_INV_GROUP_OFFSET;

        let gen_fri_group = FRI_GROUP_GEN;
        // Lyubo: Use PrimeFieldElement0::fpow
        let gen_fri_group_inv = fpow(gen_fri_group, (MAX_COSET_SIZE - 1));

        let last_val = ONE_VAL;
        let last_val_inv = ONE_VAL;

        ctx[fri_half_inv_group_ptr] = last_val_inv;
        ctx[fri_group_ptr] = last_val;
        ctx[fri_group_ptr + 1] = K_MODULUS - last_val;

        let half_coset_size = MAX_COSET_SIZE / 2;
        for i in 1..half_coset_size {
            last_val = fmul(last_val, gen_fri_group);
            last_val_inv = fmul(last_val_inv, gen_fri_group_inv);
            let idx = bit_reverse(i, FRI_MAX_STEP_SIZE - 1);

            ctx[fri_half_inv_group_ptr + idx] = last_val_inv;
            ctx[fri_group_ptr + 2 * idx] = last_val;
            ctx[fri_group_ptr + 2 * idx + 1] = K_MODULUS - last_val;
        }
    }

    fn bit_reverse(num: usize, numberOfBits: usize) -> usize {
        let mut n = num;
        let mut r = 0;
        for k in 0..numberOfBits {  
            r = (r * 2) | (n % 2);
            // Lyubo: Check if stylus division rounding of usize is like in solidity
            n = n / 2;
        }
        r
    }

    fn compute_next_layer(ctx: &[U256], proof: &[U256], channel_ptr: usize, fri_queue_ptr: usize, merkle_queue_ptr: usize, n_queries: usize, fri_ctx: usize, fri_eval_point: usize, fri_coset_size: usize) -> usize {
        let evaluations_on_coset_ptr = fri_ctx + FRI_CTX_TO_COSET_EVALUATIONS_OFFSET;

        let mut input_ptr = fri_queue_ptr;
        let input_end = input_ptr + (FRI_QUEUE_SLOT_SIZE_IN_BYTES * n_queries);
        let output_ptr = fri_queue_ptr;

        loop {
            let mut index = 0;
            let mut coset_offset = 0;
            (input_ptr, index, coset_offset) = gather_coset_inputs(ctx, proof, channel_ptr, fri_ctx + FRI_CTX_TO_FRI_GROUP_OFFSET, evaluations_on_coset_ptr, input_ptr, fri_coset_size);

            index /= fri_coset_size;

            let mut input_data = Vec::new();
            for i in evaluations_on_coset_ptr..fri_coset_size {
                input_data.extend_from_slice(&ctx[i].to_be_bytes::<32>());
            }

            ctx[merkle_queue_ptr] = index;
            ctx[merkle_queue_ptr + 1] = U256::from(COMMITMENT_MASK).bitand(uint!(keccak(&input_data).into()));
            merkle_queue_ptr += MERKLE_SLOT_SIZE_IN_BYTES;

            let (fri_value, fri_inversed_point) = transform_coset(
                ctx,
                fri_ctx + FRI_CTX_TO_FRI_HALF_INV_GROUP_OFFSET,
                evaluations_on_coset_ptr,
                coset_offset,
                fri_eval_point,
                fri_coset_size
            );

            ctx[output_ptr] = index;
            ctx[output_ptr + 1] = fri_value;
            ctx[output_ptr + 2] = fri_inversed_point;
            output_ptr += FRI_QUEUE_SLOT_SIZE_IN_BYTES;

            if input_ptr >= input_end { 
                break;
            }
        }

        return (output_ptr - fri_queue_ptr) / FRI_QUEUE_SLOT_SIZE_IN_BYTES;
    }

    fn gather_coset_inputs(ctx: &[U256], proof: &[U256], channel_ptr: usize, fri_group_ptr: usize, evaluations_on_coset_ptr: usize, fri_queue_head: usize, coset_size: usize) -> (U256, U256, U256) {
        let queue_item_idx = ctx[fri_queue_head];
        let coset_idx = queue_item_idx.bitand(U256::from(!(coset_size - 1)));
        let next_coset_idx = coset_idx.add(coset_size);

        let fri_queue = ctx[fri_queue_head + 2];
        let coset_offset = fri_queue.mul_mod(ctx[fri_group_ptr + (queue_item_idx.sub(coset_idx))], K_MODULUS);
        
        let proof_ptr = ctx[channel_ptr];

        for i in coset_idx..next_coset_idx {
            let mut field_element_ptr = proof_ptr;
            proof_ptr += 1;

            if i == queue_item_idx {
                field_element_ptr = fri_queue_head + 1;
                proof_ptr -= 1;
                fri_queue_head += FRI_QUEUE_SLOT_SIZE_IN_BYTES;
                queue_item_idx = proof[fri_queue_head];
            }

            ctx[evaluations_on_coset_ptr] = proof[field_element_ptr].rem(K_MODULUS);
            evaluations_on_coset_ptr += 1;
        }

        ctx[channel_ptr] = proof_ptr;
        (fri_queue_head, coset_idx, coset_offset)
    }

    // Lybuo: Potentially hit code size limits here
    fn transform_coset(ctx: &[U256], fri_half_inv_group_ptr: usize, evaluations_on_coset_ptr: usize, coset_offset: usize, fri_eval_point: usize, fri_coset_size: usize) -> (U256, U256) {
        if fri_coset_size == 8 {
            return transform_coset_of_size_8(ctx, fri_half_inv_group_ptr, evaluations_on_coset_ptr, coset_offset, fri_eval_point);
        } else if fri_coset_size == 4 {
            return transform_coset_of_size_4(ctx, fri_half_inv_group_ptr, evaluations_on_coset_ptr, coset_offset, fri_eval_point);
        } else if fri_coset_size == 16 {
            return transform_coset_of_size_16(ctx, fri_half_inv_group_ptr, evaluations_on_coset_ptr, coset_offset, fri_eval_point);
        }
        // In comparison to solidity, we don't have a require statement here, so we return 0s
        return (U256::ZERO, U256::ZERO);
    }

    fn transform_coset_of_size_8(ctx: &[U256], fri_half_inv_group_ptr: usize, evaluations_on_coset_ptr: usize, coset_offset: usize, fri_eval_point: usize) -> (U256, U256) {
        let mut f0 = ctx[evaluations_on_coset_ptr];
        let fri_eval_point_div_by_x = U256::from(fri_eval_point).mul_mod(coset_offset, K_MODULUS);
        let fri_eval_point_div_by_x_squared = fri_eval_point_div_by_x.mul_mod(fri_eval_point_div_by_x, K_MODULUS);
        let imaginary_unit = ctx[fri_half_inv_group_ptr + 1];

        let f1 = ctx[evaluations_on_coset_ptr + 1];
        f0 = f0.add(f1).add(fri_eval_point_div_by_x.mul_mod(f0.add(K_MODULUS.sub(f1)), K_MODULUS));

        let mut f2 = ctx[evaluations_on_coset_ptr + 2];
        let f3 = ctx[evaluations_on_coset_ptr + 3];
        f2 = f2.add(f3).add(f2.add(K_MODULUS.sub(f3)).mul_mod(fri_eval_point_div_by_x.mul_mod(imaginary_unit, K_MODULUS), K_MODULUS));

        f0 = f0.add(f2).add(fri_eval_point_div_by_x_squared.mul_mod(f0.add(K_MODULUS.sub(f2)), K_MODULUS));
        let mut f4 = ctx[evaluations_on_coset_ptr + 4];
        let fri_eval_point_div_by_x2 = fri_eval_point_div_by_x.mul_mod(ctx[fri_half_inv_group_ptr + 2], K_MODULUS);
        let f5 = ctx[evaluations_on_coset_ptr + 5];

        f4 = f4.add(f5).add(fri_eval_point_div_by_x2.mul_mod(f4.add(K_MODULUS.sub(f5)), K_MODULUS));

        let mut f6 = ctx[evaluations_on_coset_ptr + 6];
        let f7 = ctx[evaluations_on_coset_ptr + 7];
        f6 = f6.add(f7).add(f6.add(K_MODULUS.sub(f7)).mul_mod(fri_eval_point_div_by_x2.mul_mod(imaginary_unit, K_MODULUS), K_MODULUS));
        f4 = f4.add(f6).add(fri_eval_point_div_by_x2.mul_mod(fri_eval_point_div_by_x2, K_MODULUS).mul_mod(f4.add(K_MODULUS.sub(f6)), K_MODULUS));
        
        let next_layer_value = f0.add(f4).add_mod(fri_eval_point_div_by_x_squared.mul_mod(fri_eval_point_div_by_x_squared, K_MODULUS).mul_mod(f0.add(K_MODULUS_TIMES_16.sub(f4)), K_MODULUS), K_MODULUS);
        let x_inv_2 = U256::from(coset_offset).mul_mod(U256::from(coset_offset), K_MODULUS);
        let x_inv_4 = x_inv_2.mul_mod(x_inv_2, K_MODULUS);
        let next_x_inv = x_inv_4.mul_mod(x_inv_4, K_MODULUS);
        (next_layer_value, next_x_inv)
    }

    fn transform_coset_of_size_4(ctx: &[U256], fri_half_inv_group_ptr: usize, evaluations_on_coset_ptr: usize, coset_offset: usize, fri_eval_point: usize) -> (U256, U256) {
        let fri_eval_point_div_by_x = U256::from(fri_eval_point).mul_mod(coset_offset, K_MODULUS);
        let mut f0 = ctx[evaluations_on_coset_ptr];
        let f1 = ctx[evaluations_on_coset_ptr + 1];
        f0 = f0.add(f1).add(fri_eval_point_div_by_x.mul_mod(f0.add(K_MODULUS.sub(f1)), K_MODULUS));

        let mut f2 = ctx[evaluations_on_coset_ptr + 2];
        let f3 = ctx[evaluations_on_coset_ptr + 3];
        f2 = f2.add(f3).add_mod(f2.add(K_MODULUS.sub(f3)).mul_mod(ctx[fri_half_inv_group_ptr + 1].mul_mod(fri_eval_point_div_by_x, K_MODULUS), K_MODULUS), K_MODULUS);

        let mut new_x_inv = U256::from(coset_offset).mul_mod(U256::from(coset_offset), K_MODULUS);
        new_x_inv = new_x_inv.mul_mod(new_x_inv, K_MODULUS);

        let next_layer_value = f0.add(f2).add_mod(fri_eval_point_div_by_x.mul_mod(fri_eval_point_div_by_x, K_MODULUS).mul_mod(f0.add(K_MODULUS.sub(f2)), K_MODULUS), K_MODULUS);
        (next_layer_value, new_x_inv)
    }

    fn transform_coset_of_size_16(ctx: &[U256], fri_half_inv_group_ptr: usize, evaluations_on_coset_ptr: usize, coset_offset: usize, fri_eval_point: usize) -> (U256, U256) {
        let mut f0 = ctx[evaluations_on_coset_ptr];
        let fri_eval_point_div_by_x = U256::from(fri_eval_point).mul_mod(coset_offset, K_MODULUS);
        let imaginary_unit = ctx[fri_half_inv_group_ptr + 1];

        let f1 = ctx[evaluations_on_coset_ptr + 1];
        f0 = f0.add(f1).add(fri_eval_point_div_by_x.mul_mod(f0.add(K_MODULUS.sub(f1)), K_MODULUS));
        let mut f2 = ctx[evaluations_on_coset_ptr + 2];
        let f3 = ctx[evaluations_on_coset_ptr + 3];
        f2 = f2.add(f3).add(f2.add(K_MODULUS.sub(f3)).mul_mod(fri_eval_point_div_by_x.mul_mod(imaginary_unit, K_MODULUS), K_MODULUS));
        let fri_eval_point_div_by_x_squared = fri_eval_point_div_by_x.mul_mod(fri_eval_point_div_by_x, K_MODULUS);
        let fri_eval_point_div_by_x_tessed = fri_eval_point_div_by_x_squared.mul_mod(fri_eval_point_div_by_x_squared, K_MODULUS);

        f0 = f0.add(f2).add(fri_eval_point_div_by_x_squared.mul_mod(f0.add(K_MODULUS.sub(f2)), K_MODULUS));
        let mut f4 = ctx[evaluations_on_coset_ptr + 4];
        let fri_eval_point_div_by_x2 = fri_eval_point_div_by_x.mul_mod(ctx[fri_half_inv_group_ptr + 2], K_MODULUS);
        let f5 = ctx[evaluations_on_coset_ptr + 5];
        f4 = f4.add(f5).add(fri_eval_point_div_by_x2.mul_mod(f4.add(K_MODULUS.sub(f5)), K_MODULUS));

        let mut f6 = ctx[evaluations_on_coset_ptr + 6];
        let f7 = ctx[evaluations_on_coset_ptr + 7];
        f6 = f6.add(f7).add(f6.add(K_MODULUS.sub(f7)).mul_mod(fri_eval_point_div_by_x2.mul_mod(imaginary_unit, K_MODULUS), K_MODULUS));
        f4 = f4.add(f6).add(fri_eval_point_div_by_x2.mul_mod(fri_eval_point_div_by_x2, K_MODULUS).mul_mod(f4.add(K_MODULUS.sub(f6)), K_MODULUS));

        f0 = f0.add(f4).add(fri_eval_point_div_by_x_tessed.mul_mod(f0.add(K_MODULUS_TIMES_16.sub(f4)), K_MODULUS));

        let mut f8 = ctx[evaluations_on_coset_ptr + 8];
        let fri_eval_point_div_by_x4 = fri_eval_point_div_by_x.mul_mod(ctx[fri_half_inv_group_ptr + 4], K_MODULUS);
        let f9 = ctx[evaluations_on_coset_ptr + 9];
        f8 = f8.add(f9).add(fri_eval_point_div_by_x4.mul_mod(f8.add(K_MODULUS.sub(f9)), K_MODULUS));
        let mut f10 = ctx[evaluations_on_coset_ptr + 10];
        let f11 = ctx[evaluations_on_coset_ptr + 11];
        f10 = f10.add(f11).add(f10.add(K_MODULUS.sub(f11)).mul_mod(fri_eval_point_div_by_x4.mul_mod(imaginary_unit, K_MODULUS), K_MODULUS));
        f8 = f8.add(f10).add(fri_eval_point_div_by_x4.mul_mod(fri_eval_point_div_by_x4, K_MODULUS).mul_mod(f8.add(K_MODULUS.sub(f10)), K_MODULUS));
        let mut f12 = ctx[evaluations_on_coset_ptr + 12];
        let fri_eval_point_div_by_x6 = fri_eval_point_div_by_x.mul_mod(ctx[fri_half_inv_group_ptr + 6], K_MODULUS);
        let f13 = ctx[evaluations_on_coset_ptr + 13];
        f12 = f12.add(f13).add(fri_eval_point_div_by_x6.mul_mod(f12.add(K_MODULUS.sub(f13)), K_MODULUS));

        let mut f14 = ctx[evaluations_on_coset_ptr + 14];
        let f15 = ctx[evaluations_on_coset_ptr + 15];
        f14 = f14.add(f15).add(f14.add(K_MODULUS.sub(f15)).mul_mod(fri_eval_point_div_by_x6.mul_mod(imaginary_unit, K_MODULUS), K_MODULUS));
        f12 = f12.add(f14).add(fri_eval_point_div_by_x6.mul_mod(fri_eval_point_div_by_x6, K_MODULUS).mul_mod(f12.add(K_MODULUS.sub(f14)), K_MODULUS));
        f8 = f8.add(f12).add(fri_eval_point_div_by_x_tessed.mul_mod(imaginary_unit, K_MODULUS).mul_mod(f8.add(K_MODULUS_TIMES_16.sub(f12)), K_MODULUS));

        let next_layer_value = f0.add(f8).add_mod(fri_eval_point_div_by_x_tessed.mul_mod(fri_eval_point_div_by_x_tessed, K_MODULUS).mul_mod(f0.add(K_MODULUS_TIMES_16.sub(f8)), K_MODULUS), K_MODULUS);
        let x_inv_2 = U256::from(coset_offset).mul_mod(U256::from(coset_offset), K_MODULUS);
        let x_inv_4 = x_inv_2.mul_mod(x_inv_2, K_MODULUS);
        let x_inv_8 = x_inv_4.mul_mod(x_inv_4, K_MODULUS);
        let next_x_inv = x_inv_8.mul_mod(x_inv_8, K_MODULUS);
        (next_layer_value, next_x_inv)
    }

    fn verify_last_layer(ctx: &[U256], n_queries: usize) {
        let fri_last_layer_deg_bound = ctx[MM_FRI_LAST_LAYER_DEG_BOUND];
        let group_order_minus_one = fri_last_layer_deg_bound * ctx[MM_BLOW_UP_FACTOR] - 1;
        let coefs_start = ctx[MM_FRI_LAST_LAYER_PTR];

        for i in 0..n_queries {
            let mut point = ctx[MM_FRI_QUEUE + FRI_QUEUE_SLOT_SIZE * i + 2];
            // Lyubo: Use fpow from PrimeFieldElement0::fpow
            point = fpow(point, group_order_minus_one);
            require!(
                horner_eval(coefs_start, point, fri_last_layer_deg_bound) ==
                    ctx[MM_FRI_QUEUE + FRI_QUEUE_SLOT_SIZE * i + 1],
                b"Bad Last layer value."
            );
        }
    }

    fn horner_eval(ctx: &[U256], coefs_start: U256, point: U256, n_coefs: U256) -> U256 {
        let mut result = 0;
        let prime = K_MODULUS;

        require!(n_coefs.rem(8) == 0, b"Number of polynomial coefficients must be divisible by 8");
        require!(n_coefs.lt(4096), b"No more than 4096 coefficients are supported");

        let mut coefs_ptr = coefs_start.add(n_coefs);
        while coefs_ptr.gt(coefs_start) {
            coefs_ptr = coefs_ptr.sub(8);
            result = ctx[coefsPtr + 7].add(result.mul_mod(point, prime));
            result = ctx[coefsPtr + 6].add(result.mul_mod(point, prime));
            result = ctx[coefsPtr + 5].add(result.mul_mod(point, prime));
            result = ctx[coefsPtr + 4].add(result.mul_mod(point, prime));
            result = ctx[coefsPtr + 3].add(result.mul_mod(point, prime));
            result = ctx[coefsPtr + 2].add(result.mul_mod(point, prime));
            result = ctx[coefsPtr + 1].add(result.mul_mod(point, prime));
            result = ctx[coefsPtr].add(result.mul_mod(point, prime));
        }

        return result.rem(prime);
    }
}
