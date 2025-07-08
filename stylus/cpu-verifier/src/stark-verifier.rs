extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256, uint},
    crypto::keccak,
};

use macros::require;

use crate::interfaces::ICpuOods;
use crate::verifier_channel::VerifierChannel;
use crate::prime_field_element0::PrimeFieldElement0;
use crate::fri_statement_verifier::FriStatementVerifier;
use crate::merkle_statement_verifier::MerkleStatementVerifier;

const PRIME_MINUS_ONE: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000000_U256);
const COMMITMENT_MASK: U256 = uint!(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF000000000000000000000000_U256);

pub trait StarkVerifier : MerkleStatementVerifier + FriStatementVerifier {

    fn air_specific_init(public_input: &[U256]) -> Result<(Vec<U256>, U256), Vec<u8>>;

    fn oods_consistency_check(&self, ctx: &mut [U256], public_input: &[U256]) -> Result<(), Vec<u8>>;

    fn get_public_input_hash(public_input: &[U256]) -> FixedBytes<32>;

    fn get_oods_contract(&self) -> ICpuOods;

    fn verify_proof(
        &self,
        proof_params: &[U256],
        proof: &mut [U256],
        public_input: &[U256],
    ) -> Result<(), Vec<u8>> {
        let (mut ctx, fri_step_sizes) = self.init_verifier_params(public_input, proof_params)?;
        let channel_ptr = 10;

        VerifierChannel::init_channel(
            &mut ctx,
            channel_ptr,
            &Self::get_public_input_hash(public_input)
        );

        ctx[6] = VerifierChannel::read_hash(proof, &mut ctx, channel_ptr, true);

        if Self::has_interaction() {
            VerifierChannel::send_field_elements(&mut ctx, channel_ptr, 6, 352)?;
            ctx[7] = VerifierChannel::read_hash(proof, &mut ctx, channel_ptr, true);
        }

        VerifierChannel::send_field_elements(&mut ctx, channel_ptr, 1, 358)?;

        ctx[8] = VerifierChannel::read_hash(proof, &mut ctx, channel_ptr, true);

        VerifierChannel::send_field_elements(&mut ctx, channel_ptr, 1, 351)?;

        let lmm_oods_values = 359;
        for i in lmm_oods_values..lmm_oods_values + 194 {
            ctx[i] = VerifierChannel::read_field_element(proof, &mut ctx, channel_ptr, true);
        }

        self.oods_consistency_check(&mut ctx, public_input)?;

        VerifierChannel::send_field_elements(&mut ctx, channel_ptr, 1, 601)?;

        ctx[305] = VerifierChannel::read_hash(proof, &mut ctx, channel_ptr, true);
        
        let n_fri_steps = fri_step_sizes.len();
        let fri_eval_point_ptr = 295;
        for i in 1..n_fri_steps - 1 {
            VerifierChannel::send_field_elements(&mut ctx, channel_ptr, 1, fri_eval_point_ptr + i)?;
            ctx[305 + i] = VerifierChannel::read_hash(proof, &mut ctx, channel_ptr, true);
        }

        VerifierChannel::send_field_elements(&mut ctx, channel_ptr, 1, 295 + n_fri_steps - 1)?;

        Self::read_last_fri_layer(proof, &mut ctx)?;

        let proof_of_work_bits = ctx[3];
        VerifierChannel::verify_proof_of_work(proof, &mut ctx, 10, proof_of_work_bits)?;

        let count = ctx[9].to::<usize>();
        let queries_ptr = ctx[0] - U256::from(1);
        ctx[9] = VerifierChannel::send_random_queries(&mut ctx, 10, count, queries_ptr, U256::from(109), U256::from(3))?;

        self.compute_first_fri_layer(proof, &mut ctx)?;
        self.fri_verify_layers(&mut ctx, proof, &fri_step_sizes)?;
        Ok(())
    }

    fn init_verifier_params(
        &self,
        public_input: &[U256],
        proof_params: &[U256],
    ) -> Result<(Vec<U256>, Vec<U256>), Vec<u8>> {
        require!(
            proof_params.len() >= 5,
            "Invalid proof params"
        );
       
        require!(
            proof_params.len() == (5 + proof_params[4].to::<usize>()),
            "Invalid proofParams."
        );
        let log_blowup_factor = proof_params[1];
        require!(log_blowup_factor <= U256::from(16), "logBlowupFactor must be at most 16");
        require!(log_blowup_factor >= U256::from(1), "logBlowupFactor must be at least 1");
        
        let proof_of_work_bits = proof_params[2];
        require!(proof_of_work_bits <= U256::from(50), "proofOfWorkBits must be at most 50");
        require!(proof_of_work_bits >= U256::from(1), "minimum proofOfWorkBits not satisfied");
        // require!(proof_of_work_bits < num_security_bits, "Proofs may not be purely based on PoW.");

        let log_fri_last_layer_deg_bound = proof_params[3];
        require!(log_fri_last_layer_deg_bound <= U256::from(10), "logFriLastLayerDegBound must be at most 10.");

        let n_fri_steps = proof_params[4].to::<usize>();
        require!(n_fri_steps <= 10, "Too many fri steps.");
        require!(n_fri_steps > 1, "Not enough fri steps.");

        let mut fri_step_sizes: Vec<U256> = Vec::new();
        for i in 0..n_fri_steps {
            fri_step_sizes.push(proof_params[5 + i]);
        }

        let (mut ctx, log_trace_length) = Self::air_specific_init(public_input)?;
        Self::validate_fri_params(&fri_step_sizes, log_trace_length, log_fri_last_layer_deg_bound)?;

        ctx[315] = U256::from(1) << log_fri_last_layer_deg_bound;
        ctx[324] = U256::from(1) << log_trace_length;
        ctx[1] = U256::from(1) << log_blowup_factor;
        ctx[3] = proof_of_work_bits;

        let n_queries = proof_params[0];
        require!(n_queries > U256::ZERO, "Number of queries must be at least one");
        require!(n_queries <= U256::from(48), "Too many queries.");
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

        Ok((ctx, fri_step_sizes))
    }

    fn validate_fri_params(
        fri_step_sizes: &[U256],
        log_trace_length: U256,
        log_fri_last_layer_deg_bound: U256,
    ) -> Result<(), Vec<u8>> {
        require!(fri_step_sizes[0] == U256::ZERO, "Only eta0 == 0 is currently supported");

        let mut expected_log_deg_bound = log_fri_last_layer_deg_bound;
        let n_fri_steps = fri_step_sizes.len();
        for i in 1..n_fri_steps {
            let fri_step_size = fri_step_sizes[i];
            require!(fri_step_size >= U256::from(2), "Min supported fri step size is 2.");
            require!(fri_step_size <= U256::from(4), "Max supported fri step size is 4.");
            expected_log_deg_bound += fri_step_size;
        }

        require!(expected_log_deg_bound == log_trace_length, "Fri params do not match trace length");
        Ok(())
    }

    fn has_interaction() -> bool {
        true // CPUVerifier returns always true(For the purpose of our work only)
    }

    fn read_last_fri_layer(proof: &mut [U256], ctx: &mut [U256]) -> Result<(), Vec<u8>> {
        let lmm_channel = 10;
        let fri_last_layer_deg_bound = ctx[315].to::<usize>();
        let mut bad_input = U256::ZERO;

        let channel_ptr = lmm_channel;
        let last_layer_ptr = ctx[channel_ptr].to::<usize>();
        let last_layer_end = last_layer_ptr + fri_last_layer_deg_bound;
        for i in last_layer_ptr..last_layer_end {
            if proof[i] > PRIME_MINUS_ONE {
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

        require!(bad_input == U256::ZERO, "Invalid field element.");
        ctx[316] = U256::from(last_layer_ptr);
        Ok(())
    }

    // // Lyubo: Move to utils.rs
    fn u256_to_bytes(value: U256) -> FixedBytes<32> {
        let value_bytes: [u8; 32] = value.to_be_bytes();
        FixedBytes(value_bytes)
    }

    fn compute_first_fri_layer(&self, proof: &mut [U256], ctx: &mut [U256]) -> Result<(), Vec<u8>> {
        Self::adjust_query_indices_and_prepare_eval_points(ctx);
        self.read_query_responses_and_decommit(proof, ctx, 12, 9, 602, Self::u256_to_bytes(ctx[6]))?;
        if Self::has_interaction() {
            self.read_query_responses_and_decommit(proof, ctx, 12, 3, 611, Self::u256_to_bytes(ctx[7]))?;
        }
        self.read_query_responses_and_decommit(proof, ctx, 2, 2, 1178, Self::u256_to_bytes(ctx[8]))?;

        let oods_contract: ICpuOods =  self.get_oods_contract();
        let oods_result = oods_contract.compute(self, ctx.to_vec())?;
        for i in 0..oods_result.len() {
            ctx[109 + i] = oods_result[i];
        }
        
        Ok(())
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

    fn read_query_responses_and_decommit(
        &self,
        proof: &mut [U256], 
        ctx: &mut [U256], 
        n_total_columns: usize, 
        n_columns: usize, 
        mut proof_data_ptr: usize, merkle_root: FixedBytes<32>) -> Result<(), Vec<u8>> {
        require!(n_columns <= n_total_columns, "Too many columns.");

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
            let mut merkle_leaf = merkle_leaf_hash & COMMITMENT_MASK;

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
        self.verify_merkle(ctx, merkle_queue_ptr, merkle_root, n_unique_queries)?;
        Ok(())
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

