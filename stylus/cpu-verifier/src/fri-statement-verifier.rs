extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{U256, uint, FixedBytes},
    crypto::keccak,
    prelude::*,
};

use macros::require;

use crate::interfaces::IFriStatement;
use crate::verifier_channel::VerifierChannel;
use crate::prime_field_element0::PrimeFieldElement0;

pub trait FriStatementVerifier:  Sized + TopLevelStorage {

    fn get_fri_statement(&self) -> IFriStatement;

    fn fri_verify_layers(&self, ctx: &mut [U256], proof: &[U256], fri_step_sizes: &[U256]) -> Result<(), Vec<u8>> {
        let n_queries = ctx[9].to::<usize>();
        for i in 0..n_queries {
            ctx[109 + 3 * i + 1] = PrimeFieldElement0::fmul(ctx[109 + 3 * i + 1], PrimeFieldElement0::K_MONTGOMERY_R);
        }

        let mut input_data = Vec::new();
        for i in 109..n_queries * 3 + 109 {
            input_data.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        let mut input_layer_hash: U256 = uint!(keccak(&input_data).into());
        
        let n_fri_inner_layers = fri_step_sizes.len() - 1;
        let mut fri_step = 1;
        let mut sum_of_step_sizes = fri_step_sizes[1];
        let fri_statement_contract = self.get_fri_statement();
        while fri_step < n_fri_inner_layers {
            let mut data_to_hash = Vec::new();
            let output_layer_hash = VerifierChannel::read_bytes_from_ptr(proof, ctx, 10, true);
            data_to_hash.extend_from_slice(&ctx[295 + fri_step].to_be_bytes::<32>());
            data_to_hash.extend_from_slice(&fri_step_sizes[fri_step].to_be_bytes::<32>());
            data_to_hash.extend_from_slice(&input_layer_hash.to_be_bytes::<32>());
            data_to_hash.extend_from_slice(&output_layer_hash.to_be_bytes::<32>());
            data_to_hash.extend_from_slice(&ctx[305 + fri_step - 1].to_be_bytes::<32>());

            let hash: FixedBytes<32> = keccak(&data_to_hash).into();
            require!(fri_statement_contract.is_valid(self, hash)?, "INVALIDATED_FRI_STATEMENT");

            input_layer_hash = output_layer_hash;
            fri_step += 1;
            sum_of_step_sizes += fri_step_sizes[fri_step];
        }

        let mut data_to_hash = Vec::new();
        data_to_hash.extend_from_slice(&ctx[295 + fri_step].to_be_bytes::<32>());
        data_to_hash.extend_from_slice(&fri_step_sizes[fri_step].to_be_bytes::<32>());
        data_to_hash.extend_from_slice(&input_layer_hash.to_be_bytes::<32>());
        data_to_hash.extend_from_slice(&Self::compute_last_layer_hash(proof, ctx, n_queries, sum_of_step_sizes)?.to_be_bytes::<32>());
        data_to_hash.extend_from_slice(&ctx[305 + fri_step - 1].to_be_bytes::<32>());

        // Lyubo: Check the result
        let hash: FixedBytes<32> = keccak(&data_to_hash).into();
        require!(fri_statement_contract.is_valid(self, hash)?, "INVALIDATED_FRI_STATEMENT");

        Ok(())
    }

    fn compute_last_layer_hash(proof: &[U256], ctx: &mut [U256], n_points: usize, sum_of_step_sizes: U256) -> Result<U256, Vec<u8>> {
        let fri_last_layer_deg_bound = ctx[315];
        let group_order_minus_one = fri_last_layer_deg_bound * ctx[1] - U256::from(1);
        let exponent = U256::from(1) << sum_of_step_sizes;
        let mut cur_point_index = 0;
        let mut prev_query = U256::ZERO;
        let coefs_start = ctx[316].to::<usize>();
    
        for i in 0..n_points {
            let query = ctx[109 + 3 * i] >> sum_of_step_sizes;
            if query == prev_query {
                continue;
            }

            ctx[109 + 3 * cur_point_index] = query;
            prev_query = query;

            let mut point = PrimeFieldElement0::fpow(ctx[109 + 3 * i + 2], exponent);
            ctx[109 + 3 * cur_point_index + 2] = point;

            point = PrimeFieldElement0::fpow(point, group_order_minus_one);
            ctx[109 + 3 * cur_point_index + 1] = Self::horner_eval(proof, coefs_start, point, fri_last_layer_deg_bound.to::<usize>())?;

            cur_point_index += 1;
        }

        let mut data_to_hash = Vec::new();
        for i in 109..cur_point_index * 3 + 109 {
            data_to_hash.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        Ok(uint!(keccak(&data_to_hash).into()))
    }

    
    fn horner_eval(proof: &[U256], coefs_start: usize, point: U256, n_coefs: usize) -> Result<U256, Vec<u8>> {
        let mut result = U256::ZERO;
        let prime = PrimeFieldElement0::K_MODULUS;

        require!(n_coefs % 8 == 0, "Number of polynomial coefficients must be divisible by 8");
        require!(n_coefs < 4096, "No more than 4096 coefficients are supported");

        let mut coefs_ptr = coefs_start + n_coefs;
        while coefs_ptr > coefs_start {
            coefs_ptr -= 8;
            result = proof[coefs_ptr + 7] + result.mul_mod(point, prime);
            result = proof[coefs_ptr + 6] + result.mul_mod(point, prime);
            result = proof[coefs_ptr + 5] + result.mul_mod(point, prime);
            result = proof[coefs_ptr + 4] + result.mul_mod(point, prime);
            result = proof[coefs_ptr + 3] + result.mul_mod(point, prime);
            result = proof[coefs_ptr + 2] + result.mul_mod(point, prime);
            result = proof[coefs_ptr + 1] + result.mul_mod(point, prime);
            result = proof[coefs_ptr] + result.mul_mod(point, prime);
        }
        
        Ok(result % prime)
    }
}