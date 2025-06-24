use crate::consts::{input_offsets::*, memory_offsets::*, stark_parameters::*};
use crate::require;
use stylus_sdk::alloy_primitives::{uint, U256};
use stylus_sdk::alloy_sol_types::sol;
use stylus_sdk::prelude::SolidityError;

use crate::public_memory_offsets::{offset_page_size, public_input_length};

pub struct MerkleStatementVerifier {}

// Lyubo: Add merkle_statement_contract into the storage of the contract
impl MerkleStatementVerifier {

    fn verify_merkle(
        &self,
        ctx: &[FixedBytes<32>],
        queue_ptr: &usize,
        root: &FixedBytes<32>,
        n: &U256,
    ) -> FixedBytes<32> {
        require!(n <= MAX_N_MERKLE_VERIFIER_QUERIES, b"TOO_MANY_MERKLE_QUERIES");

        let data_to_hash_ptr_start = 0
        let data_to_hash_ptr_cur = 0;
        let que_end_ptr = queue_ptr + n * 2;

        let mut input_data = Vec::new();
        for i in queue_ptr..que_end_ptr {
            input_data.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        input_data.extend_from_slice(&root);

        let statement = keccak(&input_data).into();
        require!(merkle_statement_contract.is_valid(statement), b"INVALIDATED_MERKLE_STATEMENT");
        root
    }
}
