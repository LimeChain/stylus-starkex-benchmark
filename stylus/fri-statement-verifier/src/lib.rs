#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use utils::{prime_field_element0::PrimeFieldElement0, require};

#[path = "interfaces.rs"]
pub mod interfaces;
use crate::interfaces::{ICpuOods, IFriStatementVerifier};

use alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{uint, Address, FixedBytes, U256},
    crypto::keccak,
    prelude::*,
};

sol! {
    event FriVerified(
        uint256[] ctx
    );
}

sol_storage! {
    #[entrypoint]
    pub struct FriStatementVerifier {
        address oods;
        address fri_statement;
        address merkle_statement;
    }
}

// const COMMITMENT_MASK: U256 = uint!(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF000000000000000000000000_U256);

#[public]
impl FriStatementVerifier {
    #[inline]
    fn init(&mut self, oods: Address, fri_statement: Address, merkle_statement: Address) {
        self.oods.set(oods);
        self.fri_statement.set(fri_statement);
        self.merkle_statement.set(merkle_statement);
    }

    #[inline]
    fn verify(
        &mut self,
        mut proof: Vec<U256>,
        mut ctx: Vec<U256>,
        fri_step_sizes: Vec<U256>,
    ) -> Result<Vec<U256>, Vec<u8>> {
        Self::adjust_query_indices_and_prepare_eval_points(&mut ctx);

        let val1: FixedBytes<32> = FixedBytes(ctx[6].to_be_bytes());
        let val2: FixedBytes<32> = FixedBytes(ctx[7].to_be_bytes());
        let val3: FixedBytes<32> = FixedBytes(ctx[8].to_be_bytes());
        self.read_query_responses_and_decommit(&mut proof, &mut ctx, 12, 9, 602, val1)?;
        self.read_query_responses_and_decommit(&mut proof, &mut ctx, 12, 3, 611, val2)?;
        self.read_query_responses_and_decommit(&mut proof, &mut ctx, 2, 2, 1178, val3)?;

        let oods_contract: ICpuOods = ICpuOods {
            address: self.oods.get(),
        };
        let oods_result: Vec<U256> = oods_contract.compute(&mut *self, ctx.to_vec())?;
        for i in 0..oods_result.len() {
            ctx[109 + i] = oods_result[i];
        }

        self.fri_verify_layers(&mut ctx, &proof, &fri_step_sizes)?;
        Ok(ctx)
    }
}

impl FriStatementVerifier {
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
            ctx[eval_points_ptr] = PrimeFieldElement0::expmod(
                eval_domain_generator,
                PrimeFieldElement0::bit_reverse(query_idx, log_eval_domain_size),
                PrimeFieldElement0::K_MODULUS,
            );
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
        mut proof_data_ptr: usize,
        merkle_root: FixedBytes<32>,
    ) -> Result<(), Vec<u8>> {
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
            let mut merkle_leaf = merkle_leaf_hash
                & uint!(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF000000000000000000000000_U256);

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

    fn read_bytes_from_ptr(proof: &[U256], ctx: &mut [U256], channel_ptr: usize) -> U256 {
        let proof_ptr = ctx[channel_ptr];
        let val = Self::read_ptr(proof, proof_ptr.to::<usize>(), 8);
        ctx[channel_ptr] = proof_ptr + U256::from(32);

        let mut input_data = Vec::new();
        input_data.extend_from_slice(&(ctx[channel_ptr + 1] + U256::from(1)).to_be_bytes::<32>());
        input_data.extend_from_slice(&val.to_be_bytes::<32>());
        ctx[channel_ptr + 1] = uint!(keccak(&input_data).into());
        ctx[channel_ptr + 2] = U256::ZERO;

        val
    }

    fn fri_verify_layers(
        &self,
        ctx: &mut [U256],
        proof: &[U256],
        fri_step_sizes: &[U256],
    ) -> Result<(), Vec<u8>> {
        let n_queries = ctx[9].to::<usize>();
        for i in 0..n_queries {
            ctx[109 + 3 * i + 1] =
                PrimeFieldElement0::fmul(ctx[109 + 3 * i + 1], PrimeFieldElement0::K_MONTGOMERY_R);
        }

        let mut input_data = Vec::new();
        for i in 109..n_queries * 3 + 109 {
            input_data.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        let mut input_layer_hash: U256 = uint!(keccak(&input_data).into());

        let n_fri_inner_layers = fri_step_sizes.len() - 1;
        let mut fri_step = 1;
        let mut sum_of_step_sizes = fri_step_sizes[1];
        let fri_statement_contract: IFriStatementVerifier = IFriStatementVerifier {
            address: self.fri_statement.get(),
        };
        while fri_step < n_fri_inner_layers {
            let mut data_to_hash = Vec::new();
            let output_layer_hash = Self::read_bytes_from_ptr(proof, ctx, 10);
            data_to_hash.extend_from_slice(&ctx[295 + fri_step].to_be_bytes::<32>());
            data_to_hash.extend_from_slice(&fri_step_sizes[fri_step].to_be_bytes::<32>());
            data_to_hash.extend_from_slice(&input_layer_hash.to_be_bytes::<32>());
            data_to_hash.extend_from_slice(&output_layer_hash.to_be_bytes::<32>());
            data_to_hash.extend_from_slice(&ctx[305 + fri_step - 1].to_be_bytes::<32>());

            let hash: FixedBytes<32> = keccak(&data_to_hash).into();
            require!(
                fri_statement_contract.is_valid(self, hash)?,
                "INVALIDATED_FRI_STATEMENT"
            );

            input_layer_hash = output_layer_hash;
            fri_step += 1;
            sum_of_step_sizes += fri_step_sizes[fri_step];
        }

        let mut data_to_hash = Vec::new();
        data_to_hash.extend_from_slice(&ctx[295 + fri_step].to_be_bytes::<32>());
        data_to_hash.extend_from_slice(&fri_step_sizes[fri_step].to_be_bytes::<32>());
        data_to_hash.extend_from_slice(&input_layer_hash.to_be_bytes::<32>());
        data_to_hash.extend_from_slice(
            &Self::compute_last_layer_hash(proof, ctx, n_queries, sum_of_step_sizes)?
                .to_be_bytes::<32>(),
        );
        data_to_hash.extend_from_slice(&ctx[305 + fri_step - 1].to_be_bytes::<32>());

        let hash: FixedBytes<32> = keccak(&data_to_hash).into();
        require!(
            fri_statement_contract.is_valid(self, hash)?,
            "INVALIDATED_FRI_STATEMENT"
        );

        Ok(())
    }

    fn compute_last_layer_hash(
        proof: &[U256],
        ctx: &mut [U256],
        n_points: usize,
        sum_of_step_sizes: U256,
    ) -> Result<U256, Vec<u8>> {
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
            ctx[109 + 3 * cur_point_index + 1] = Self::horner_eval(
                proof,
                coefs_start,
                point,
                fri_last_layer_deg_bound.to::<usize>(),
            )?;

            cur_point_index += 1;
        }

        let mut data_to_hash = Vec::new();
        for i in 109..cur_point_index * 3 + 109 {
            data_to_hash.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        Ok(uint!(keccak(&data_to_hash).into()))
    }

    fn horner_eval(
        proof: &[U256],
        coefs_start: usize,
        point: U256,
        n_coefs: usize,
    ) -> Result<U256, Vec<u8>> {
        let mut result = U256::ZERO;
        let prime = PrimeFieldElement0::K_MODULUS;

        require!(
            n_coefs % 8 == 0,
            "Number of polynomial coefficients must be divisible by 8"
        );
        require!(
            n_coefs < 4096,
            "No more than 4096 coefficients are supported"
        );

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

    fn verify_merkle(
        &self,
        ctx: &[U256],
        queue_ptr: usize,
        root: FixedBytes<32>,
        n: usize,
    ) -> Result<FixedBytes<32>, Vec<u8>> {
        require!(n <= 128, "TOO_MANY_MERKLE_QUERIES");

        let que_end_ptr = queue_ptr + n * 2;
        let mut input_data = Vec::new();
        for i in queue_ptr..que_end_ptr {
            input_data.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        input_data.extend_from_slice(&root.as_slice());

        let statement: FixedBytes<32> = keccak(&input_data).into();
        // let merkle_contract: Address = self.merkle_statement.get();
        // require!(merkle_contract.is_valid(self, statement)?, "INVALIDATED_MERKLE_STATEMENT");
        Ok(root)
    }
}
