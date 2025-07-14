extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256, uint},
    crypto::keccak,
    prelude::*,
};

use utils::require;
use crate::interfaces::IInitVerifier;
use crate::interfaces::IFriStatementVerifier;
use crate::verifier_channel::VerifierChannel;

const PRIME_MINUS_ONE: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000000_U256);

pub trait StarkVerifier : HostAccess + Sized + TopLevelStorage {

    fn oods_consistency_check(&mut self, ctx: &mut [U256], public_input: &[U256]) -> Result<(), Vec<u8>>;

    fn get_public_input_hash(public_input: &[U256]) -> FixedBytes<32>;

    fn get_init_verifier(&self) -> IInitVerifier;

    fn get_fri_statement_verifier(&self) -> IFriStatementVerifier;

    fn verify_proof(
        &mut self,
        proof_params: &[U256],
        proof: &mut [U256],
        public_input: &[U256],
    ) -> Result<Vec<U256>, Vec<u8>> {
        let init_verifier = self.get_init_verifier();
        let (mut ctx, fri_step_sizes) = init_verifier.init_verifier_params(&mut *self, public_input.to_vec(), proof_params.to_vec())?;
        
        
        let channel_ptr = 10;
        VerifierChannel::init_channel(
            &mut ctx,
            channel_ptr,
            &Self::get_public_input_hash(public_input)
        );
        
        ctx[6] = VerifierChannel::read_hash(proof, &mut ctx, channel_ptr, true);
        
        VerifierChannel::send_field_elements(&mut ctx, channel_ptr, 6, 352)?;
        ctx[7] = VerifierChannel::read_hash(proof, &mut ctx, channel_ptr, true);
        
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
        
        let fri_statement_verifier = self.get_fri_statement_verifier();
        let result = fri_statement_verifier.verify(&mut *self, proof.to_vec(), ctx.to_vec(), fri_step_sizes.to_vec())?;
        Ok(result)
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
}

