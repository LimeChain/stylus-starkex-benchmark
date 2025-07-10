// #![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256},
    crypto::keccak,
    prelude::*,
};

use macros::require;
use crate::interfaces::IMerkleStatement;

pub trait MerkleStatementVerifier: Sized + TopLevelStorage {

    fn get_merkle_statement(&self) -> IMerkleStatement;

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
        let merkle_contract: IMerkleStatement =  self.get_merkle_statement();
        // require!(merkle_contract.is_valid(self, statement)?, "INVALIDATED_MERKLE_STATEMENT");
        Ok(root)
    }
}


// sol_storage! {
//     #[entrypoint]
//     pub struct Test {
//         address merkle_statement;
//     }
// }


// impl MerkleStatementVerifier for Test {
//     fn get_merkle_statement(&self) -> IMerkleStatement {
//         IMerkleStatement { address: self.merkle_statement.get() }
//     }
// }

// #[public]
// impl Test {

//     pub fn test(&self) {
//         // self._test();
//         let ctx = [U256::ZERO; 1277];
//         let queue_ptr = 10;
//         let root = FixedBytes::from([0; 32]);
//         let n = 10;
//         self.verify_merkle(&ctx, queue_ptr, root, n);
//     }
// }

// impl Test {

//     pub fn _test(&self) {
//         let ctx = [U256::ZERO; 1277];
//         let queue_ptr = 10;
//         let root = FixedBytes::from([0; 32]);
//         let n = 10;
//         self.verify_merkle(&ctx, queue_ptr, root, n);
//     }
// }
