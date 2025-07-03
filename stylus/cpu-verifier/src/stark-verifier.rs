extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{address, FixedBytes, U256, uint},
    crypto::keccak,
    call::{static_call, Call},
};

#[path = "prime-field-element0.rs"]
mod prime_field_element0;
use prime_field_element0::PrimeFieldElement0;

#[path = "merkle-statement-verifier.rs"]
mod merkle_statement_verifier;
use merkle_statement_verifier::MerkleStatementVerifier;

pub trait StarkVerifier {
    const PRIME_MINUS_ONE: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000000_U256);
    const COMMITMENT_MASK: U256 = uint!(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF000000000000000000000000_U256);
    fn air_specific_init(public_input: &[U256]) -> (Vec<U256>, U256);

    fn init_verifier_params(
        public_input: &[U256],
        proof_params: &[U256],
    ) -> (Vec<U256>, Vec<U256>) {
        // require!(
        //     proof_params.len() >= PROOF_PARAMS_FRI_STEPS_OFFSET,
        //     "Invalid proof params"
        // );
       
        // require!(
        //     proof_params.len() == (PROOF_PARAMS_FRI_STEPS_OFFSET + proof_params[PROOF_PARAMS_N_FRI_STEPS_OFFSET]),
        //     "Invalid proofParams."
        // );
        let log_blowup_factor = proof_params[1];
        // require!(log_blowup_factor <= 16, "logBlowupFactor must be at most 16");
        // require!(log_blowup_factor >= 1, "logBlowupFactor must be at least 1");
        
        let proof_of_work_bits = proof_params[2];
        // require!(proof_of_work_bits <= 50, "proofOfWorkBits must be at most 50");
        // require!(proof_of_work_bits >= 1, "minimum proofOfWorkBits not satisfied");
        // require!(proof_of_work_bits < num_security_bits, "Proofs may not be purely based on PoW.");

        let log_fri_last_layer_deg_bound = proof_params[3];
        // require!(log_fri_last_layer_deg_bound <= 10, "logFriLastLayerDegBound must be at most 10.");

        let n_fri_steps = proof_params[4].to::<usize>();
        // require!(n_fri_steps <= MAX_FRI_STEPS, "Too many fri steps.");
        // require!(n_fri_steps > 1, "Not enough fri steps.");

        let mut fri_step_sizes: Vec<U256> = Vec::new();
        for i in 0..n_fri_steps {
            fri_step_sizes.push(proof_params[5 + i]);
        }

        let (mut ctx, log_trace_length) = Self::air_specific_init(public_input);
        Self::validate_fri_params(&fri_step_sizes, log_trace_length, log_fri_last_layer_deg_bound);

        ctx[315] = U256::from(1) << log_fri_last_layer_deg_bound;
        ctx[324] = U256::from(1) << log_trace_length;
        ctx[1] = U256::from(1) << log_blowup_factor;
        ctx[3] = proof_of_work_bits;

        let n_queries = proof_params[0];
        // require!(n_queries > 0, "Number of queries must be at least one");
        // require!(n_queries <= MAX_N_QUERIES, "Too many queries.");
        // require!(
        //     n_queries * log_blowup_factor + proof_of_work_bits >= num_security_bits,
        //     "Proof params do not satisfy security requirements."
        // );

        ctx[9] = n_queries;
        ctx[2] = log_trace_length + log_blowup_factor;
        ctx[0] = U256::from(1) << ctx[2];

        let gen_eval_domain = PrimeFieldElement0::fpow(U256::from(3), (PrimeFieldElement0::K_MODULUS - U256::from(1)) / ctx[0]);
        ctx[4] = gen_eval_domain;
        ctx[350] = PrimeFieldElement0::fpow(gen_eval_domain, ctx[1]);

        (ctx, fri_step_sizes)
    }


    fn validate_fri_params(
        fri_step_sizes: &[U256],
        log_trace_length: U256,
        log_fri_last_layer_deg_bound: U256,
    ) {
        // require(fri_step_sizes[0] == U256::ZERO, "Only eta0 == 0 is currently supported");

        let mut expected_log_deg_bound = log_fri_last_layer_deg_bound;
        let n_fri_steps = fri_step_sizes.len();
        for i in 1..n_fri_steps {
            let fri_step_size = fri_step_sizes[i];
            // require(fri_step_size >= U256::from(2), "Min supported fri step size is 2.");
            // require(fri_step_size <= U256::from(4), "Max supported fri step size is 4.");
            expected_log_deg_bound += fri_step_size;
        }

        // require(expected_log_deg_bound == log_trace_length, "Fri params do not match trace length");
    }

    fn has_interaction() -> bool {
        true // CPUVerifier returns always true(For the purpose of our work only)
    }

    // fn verify_proof(
    //     &self,
    //     proof_params: &[U256],
    //     proof: &[U256],
    //     public_input: &[U256],
    // ) -> Result<(), VerifierError> {
    //     (let ctx, let friStepSizes) = self.init_verifier_params(public_input, proof_params)?;
    //     let channel_ptr = get_channel_ptr();

    //     init_channel(
    //         &ctx,
    //         &channel_ptr,
    //         &get_public_input_hash(public_input)
    //     );

    //     ctx[MM_TRACE_COMMITMENT] = read_hash(&proof, &ctx, &channel_ptr, true);

    //     if Self::has_interaction() {
    //         send_field_elements(&ctx, &channel_ptr, get_n_interaction_elements(), get_mm_interaction_elements());
    //         ctx[MM_TRACE_COMMITMENT + 1] = read_hash(&proof, &ctx, &channel_ptr, true);
    //     }

    //     send_field_elements(&ctx, &channel_ptr, 1, MM_COMPOSITION_ALPHA);

    //     ctx[MM_OODS_COMMITMENT] = read_hash(&proof, &ctx, &channel_ptr, true);

    //     send_field_elements(&ctx, &channel_ptr, 1, MM_OODS_POINT);

    //     let lmm_oods_values = get_mm_oods_values();
    //     for i in lmm_oods_values..lmm_oods_values + get_n_oods_values() {
    //         ctx[i] = read_field_element(&proof, &ctx, &channel_ptr, true);
    //     }

    //     oods_consistency_check(&ctx);

    //     send_field_elements(&ctx, &channel_ptr, 1, MM_OODS_ALPHA);

    //     ctx[MM_FRI_COMMITMENTS] = read_hash(&proof, &ctx, &channel_ptr, true);
        
    //     let n_fri_steps = get_fri_step_sizes(&ctx).len();
    //     let fri_eval_point_ptr = 295 // Lyubo: Use MM_FRI_EVAL_POINTS;
    //     for i in 1..n_fri_steps - 1 {
    //         send_field_elements(&ctx, &channel_ptr, 1, fri_eval_point_ptr + i);
    //         ctx[MM_FRI_COMMITMENTS + i] = read_hash(&proof, &ctx, &channel_ptr, true);
    //     }

    //     // Send last random FRI evaluation point.
    //     send_field_elements(&ctx, &channel_ptr, 1, 295 + n_fri_steps - 1);

    //     // Read FRI last layer commitment.
    //     read_last_fri_layer(&ctx);

    //     // Generate queries.
    //     verify_proof_of_work(&ctx, &channel_ptr, ctx[MM_PROOF_OF_WORK_BITS]);

    //     // Lyubo:FRI_QUEUE_SLOT_SIZE_IN_BYTES should be FRI_QUEUE_SLOT_SIZE and define it
    //     ctx[MM_N_UNIQUE_QUERIES] = send_random_queries(&ctx, &channel_ptr, ctx[MM_N_UNIQUE_QUERIES], ctx[MM_EVAL_DOMAIN_SIZE] - 1, MM_FRI_QUEUE, 3);

    //     self.compute_first_fri_layer(&proof, &ctx);

    //     fri_verify_layers(&ctx, &proof, &friStepSizes);
    //     Ok(())
    // }

    // Lyubo: Consider if stylus requires self in the function signature
    fn read_last_fri_layer(proof: &mut [U256], ctx: &mut [U256]) {
        let lmm_channel = 10;
        let fri_last_layer_deg_bound = ctx[315].to::<usize>();
        let mut bad_input = U256::ZERO;

        let channel_ptr = lmm_channel;
        let last_layer_ptr = ctx[channel_ptr].to::<usize>();
        let last_layer_end = last_layer_ptr + fri_last_layer_deg_bound;
        for i in last_layer_ptr..last_layer_end {
            if proof[i] > Self::PRIME_MINUS_ONE {
                bad_input |= U256::from(1);
            } else {
                bad_input |= U256::ZERO;
            }
        }

        let new_digest_ptr = last_layer_ptr - 1;
        let digest_ptr = channel_ptr + 1;
        proof[new_digest_ptr] = ctx[digest_ptr] + U256::from(1);

        let mut input_data = Vec::new();
        for i in new_digest_ptr..new_digest_ptr + fri_last_layer_deg_bound + 1 {
            input_data.extend_from_slice(&proof[i].to_be_bytes::<32>());
        }

        ctx[digest_ptr] = uint!(keccak(&input_data).into());
        ctx[channel_ptr + 2] = U256::ZERO;
        ctx[channel_ptr] = U256::from(last_layer_end);

        // require!(bad_input == U256::ZERO, "Invalid field element.");
        ctx[316] = U256::from(last_layer_ptr);
        // Ok(())
    }

    // // Lyubo: Move to utils.rs
    fn u256_to_bytes(value: U256) -> FixedBytes<32> {
        let value_bytes: [u8; 32] = value.to_be_bytes();
        FixedBytes(value_bytes)
    }

    fn compute_first_fri_layer(proof: &mut [U256], ctx: &mut [U256]) {
        Self::adjust_query_indices_and_prepare_eval_points(ctx);
        Self::read_query_responses_and_decommit(proof, ctx, 12, 9, 602, Self::u256_to_bytes(ctx[6]));
        if Self::has_interaction() {
            Self::read_query_responses_and_decommit(proof, ctx, 12, 3, 611, Self::u256_to_bytes(ctx[7]));
        }
        Self::read_query_responses_and_decommit(proof, ctx, 2, 2, 1178, Self::u256_to_bytes(ctx[8]));

        // Lyubo: How to handler reverts? "?" sign?
        // ctx[MM_FRI_QUEUE] = U256::from_be_slice(&static_call(
        //     Call::new_in(self), 
        //     oodsContractAddress,
        //     &ctx
        // ));
    }

    fn adjust_query_indices_and_prepare_eval_points(ctx: &mut [U256]) {
        let n_unique_queries = ctx[9].to::<usize>();
        let fri_queue = 109;
        let fri_queue_end = fri_queue + n_unique_queries * 3;

        let mut eval_points_ptr = 553;
        let log_eval_domain_size = ctx[2].to::<usize>();
        let eval_domain_size = ctx[0];
        let eval_domain_generator = ctx[4];

        let mut i = fri_queue;
        while i < fri_queue_end {
            let query_idx = ctx[i];
            let adjusted_query_idx = query_idx + eval_domain_size;
            ctx[i] = adjusted_query_idx;
            ctx[eval_points_ptr] = PrimeFieldElement0::expmod(eval_domain_generator, PrimeFieldElement0::bit_reverse(query_idx, log_eval_domain_size), PrimeFieldElement0::K_MODULUS);
            eval_points_ptr += 1;
            i += 3;
        }
    }

    // // Lyubo: pass proof as it work with proof data, not only the ctx
    fn read_query_responses_and_decommit(proof: &mut [U256], ctx: &mut [U256], n_total_columns: usize, n_columns: usize, mut proof_data_ptr: usize, merkle_root: FixedBytes<32>) {
        // require!(n_columns <= n_total_columns, b"Too many columns.");

        let n_unique_queries = ctx[9].to::<usize>();
        let channel_ptr = 10;
        let fri_queue = 109;
        let fri_queue_end = fri_queue + n_unique_queries * 3;
        let merkle_queue_ptr = 13;
        let row_size = n_columns * 32;
        let proof_data_skip_bytes = n_total_columns - n_columns;

        let mut proof_ptr = ctx[channel_ptr].to::<usize>();
        let mut merkle_ptr = merkle_queue_ptr;

        let mut i = fri_queue;
        while i < fri_queue_end {
            let mut j = proof_ptr;
            let mut input_data = Vec::new();
            while j < proof_ptr + row_size {
                ctx[proof_data_ptr] = Self::read_ptr(proof, j, 8);
                input_data.extend_from_slice(&ctx[proof_data_ptr].to_be_bytes::<32>());
                proof_data_ptr += 1;
                j += 32;
            }

            let merkle_leaf_hash: U256 = keccak(&input_data).into();
            let mut merkle_leaf = merkle_leaf_hash & Self::COMMITMENT_MASK;

            if row_size == 32 {
                merkle_leaf = Self::read_ptr(proof, proof_ptr, 8);
            } 

            ctx[merkle_ptr] = ctx[i];
            ctx[merkle_ptr + 1] = merkle_leaf;
            merkle_ptr += 2;

            i += 3;
            proof_ptr += row_size;
            proof_data_ptr += proof_data_skip_bytes;
        }

        ctx[channel_ptr] = U256::from(proof_ptr);
        MerkleStatementVerifier::verify_merkle(ctx, merkle_queue_ptr, merkle_root, n_unique_queries);
    }

    fn read_ptr(proof: &[U256], ptr: usize, offset: usize) -> U256 {
        let element_index = ptr / 32;
        
        if ptr % 32 == 0 {
            proof[element_index]
        } else {
            let bit_shift = offset * 8;
            let element1 = proof[element_index] << bit_shift;
            let element2 = proof[element_index + 1] >> (256 - bit_shift);
            element1 | element2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod test_constants;

    // #[motsu::test]
    // fn test_read_last_fri_layer() {
    //     let mut proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_read_last_fri_layer();
    //     StarkVerifier::read_last_fri_layer(&mut proof, &mut ctx);

    //     assert_eq!(ctx[10], uint!(268_U256));
    //     assert_eq!(ctx[11], uint!(101063039785234930674416911940782140361807536835453250352760633033315826439229_U256));
    //     assert_eq!(ctx[316], uint!(204_U256));
    // }

    // Lyubo: Should fix this test
    // #[motsu::test]
    // fn test_compute_first_fri_layer() {
    //     let mut proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
    //     StarkVerifier::compute_first_fri_layer(&mut proof, &mut ctx);
    // }

    // #[motsu::test]
    // fn test_adjust_query_indices_and_prepare_eval_points() {
    //     let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
    //     StarkVerifier::adjust_query_indices_and_prepare_eval_points(&mut ctx);
    //     assert_eq!(ctx[553], uint!(3515892385904170702434114719646176958489529091479346127319408828731691841909_U256));
    //     assert_eq!(ctx[109], uint!(4818245268_U256));
    //     assert_eq!(ctx[139], uint!(8285752452_U256));
    // }

    // // Lyubo: Finish this test with asserts
    // #[motsu::test]
    // fn test_read_query_responses_and_decommit() {
    //     let mut proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
    //     StarkVerifier::adjust_query_indices_and_prepare_eval_points(&mut ctx);

    //     ctx[10] = U256::from(8584); // proof pointer
    //     let merkle_root = StarkVerifier::u256_to_bytes(ctx[6]);
    //     StarkVerifier::read_query_responses_and_decommit(&mut proof, &mut ctx, 12, 9, 602, merkle_root);
    // }

}
