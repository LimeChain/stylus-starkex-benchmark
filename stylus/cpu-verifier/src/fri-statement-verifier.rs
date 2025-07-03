extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{U256, uint},
    crypto::keccak,
};

#[path = "prime-field-element0.rs"]
mod prime_field_element0;
use prime_field_element0::PrimeFieldElement0;

#[path = "verifier-channel.rs"]
mod verifier_channel;
use verifier_channel::VerifierChannel;

pub struct FriStatementVerifier {}

// Lyubo: Add merkle_statement_contract into the storage of the contract
impl FriStatementVerifier {

    fn fri_verify_layers(ctx: &mut [U256], proof: &[U256], fri_step_sizes: &[U256]) {
        let channel_ptr = 10;
        let n_queries = ctx[9].to::<usize>();
        for i in 0..n_queries {
            ctx[109 + 3 * i + 1] = PrimeFieldElement0::fmul(ctx[109 + 3 * i + 1], PrimeFieldElement0::K_MONTGOMERY_R);
        }

        let fri_queue = ctx[109];
        let mut input_data = Vec::new();
        for i in 109..n_queries * 3 + 109 {
            input_data.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        let mut input_layer_hash = uint!(keccak(&input_data).into());
        
        let n_fri_inner_layers = fri_step_sizes.len() - 1;
        let mut fri_step = 1;
        let mut sum_of_step_sizes = fri_step_sizes[1];
        let mut data_to_hash: [U256; 5] = [U256::ZERO; 5];
        while fri_step < n_fri_inner_layers {
            let output_layer_hash = VerifierChannel::read_bytes_from_ptr(proof, ctx, 10, true);
            data_to_hash[0] = ctx[295 + fri_step];
            data_to_hash[1] = fri_step_sizes[fri_step];
            data_to_hash[2] = input_layer_hash;
            data_to_hash[3] = output_layer_hash;
            data_to_hash[4] = ctx[305 + fri_step - 1];

            // require!(friStatementContract.isValid(keccak256(abi.encodePacked(dataToHash))), "INVALIDATED_FRI_STATEMENT");

            input_layer_hash = output_layer_hash;
            fri_step += 1;
            sum_of_step_sizes += fri_step_sizes[fri_step];
        }

        data_to_hash[0] = ctx[295 + fri_step];
        data_to_hash[1] = fri_step_sizes[fri_step];
        data_to_hash[2] = input_layer_hash;
        data_to_hash[3] = FriStatementVerifier::compute_last_layer_hash(proof, ctx, n_queries, sum_of_step_sizes);
        data_to_hash[4] = ctx[305 + fri_step - 1];

        // require(
        //     friStatementContract.isValid(keccak256(abi.encodePacked(dataToHash))),
        //     "INVALIDATED_FRI_STATEMENT"
        // );
    }

    fn compute_last_layer_hash(proof: &[U256], ctx: &mut [U256], n_points: usize, sum_of_step_sizes: U256) -> U256 {
        let fri_last_layer_deg_bound = ctx[315];
        let group_order_minus_one = fri_last_layer_deg_bound * ctx[1] - U256::from(1);
        let exponent = U256::from(1) << sum_of_step_sizes;
        let mut cur_point_index = 0;
        let mut prev_query = U256::ZERO;
        let mut coefs_start = ctx[316].to::<usize>();
    
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
            ctx[109 + 3 * cur_point_index + 1] = FriStatementVerifier::horner_eval(proof, coefs_start, point, fri_last_layer_deg_bound.to::<usize>());

            cur_point_index += 1;
        }

        let mut data_to_hash = Vec::new();
        for i in 109..cur_point_index * 3 + 109 {
            data_to_hash.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        uint!(keccak(&data_to_hash).into())
    }

    
    fn horner_eval(proof: &[U256], coefs_start: usize, point: U256, n_coefs: usize) -> U256 {
        let mut result = U256::ZERO;
        let prime = PrimeFieldElement0::K_MODULUS;

        // require!(n_coefs % 8 == 0, "Number of polynomial coefficients must be divisible by 8");
        // require!(n_coefs < 4096, "No more than 4096 coefficients are supported");

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
        
        return result % prime;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    mod test_constants;

    // #[motsu::test]
    // fn test_fri_verify_layers() {
    //     let mut fri_step_sizes = Vec::new();
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(2));
    //     let proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_fri_verify_layers();
    //     FriStatementVerifier::fri_verify_layers(&mut ctx, &proof, &fri_step_sizes);
    // }

    // #[motsu::test]
    // fn test_compute_last_layer_hash() {
    //     let proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_compute_last_layer_hash();
    //     let res = FriStatementVerifier::compute_last_layer_hash(&proof, &mut ctx, 11, U256::from(20));
    //     assert_eq!(res, uint!(16162843800108123221986333459199870243499406093086027266637045595326264638953_U256));
    // }

    // #[motsu::test]
    // fn test_horner_eval() {
    //     let proof = test_constants::get_proof();
    //     let res = FriStatementVerifier::horner_eval(&proof, 204, uint!(261724642622844706275344931861363185671055404258368687742740457067613420050_U256), 64);
    //     assert_eq!(res, uint!(2139028133873562710792122920124178712162573015562878092221167762764054446737_U256));
    // }
}

