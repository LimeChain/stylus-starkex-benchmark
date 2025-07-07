#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;
use alloc::vec::Vec;

use alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256, uint, Address},
    crypto::keccak,
    prelude::*,
};

use macros::require;
// use crate::interfaces::IMerkleStatement;

sol_storage! {
    #[entrypoint]
    pub struct MerkleStatementVerifier {
        address merkle_statement;
    }
}

sol_interface! {
    interface IMerkleStatement {
        function is_valid(bytes32 statement) external view returns(bool);
    }
}

#[public]
impl MerkleStatementVerifier {

    #[constructor]
    pub fn constructor(&mut self, merkle_statement_contract: Address) {
        self.merkle_statement.set(merkle_statement_contract);
    }
}

impl MerkleStatementVerifier {

    pub fn verify_merkle(
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

        let statement: U256 = uint!(keccak(&input_data).into());
        let merkle_contract: IMerkleStatement =  IMerkleStatement { address: self.merkle_statement.get() };
        require!(merkle_contract.is_valid(self, statement.into())?, "INVALIDATED_MERKLE_STATEMENT");
        Ok(root)
    }
}
