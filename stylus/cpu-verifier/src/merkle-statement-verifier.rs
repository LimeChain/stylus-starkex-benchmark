extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256, uint},
    crypto::keccak
};

pub struct MerkleStatementVerifier {}

// Lyubo: Add merkle_statement_contract into the storage of the contract
impl MerkleStatementVerifier {

    pub fn verify_merkle(
        ctx: &[U256],
        queue_ptr: usize,
        root: FixedBytes<32>,
        n: usize,
    ) -> FixedBytes<32> {
        // require!(n <= MAX_N_MERKLE_VERIFIER_QUERIES, "TOO_MANY_MERKLE_QUERIES");

        let data_to_hash_ptr_start = 0;
        let data_to_hash_ptr_cur = 0;
        let que_end_ptr = queue_ptr + n * 2;

        let mut input_data = Vec::new();
        for i in queue_ptr..que_end_ptr {
            input_data.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        input_data.extend_from_slice(&root.as_slice());

        let statement: U256 = uint!(keccak(&input_data).into());
        // require!(merkle_statement_contract.is_valid(statement), b"INVALIDATED_MERKLE_STATEMENT");
        root
    }
}
