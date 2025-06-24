//!
//! The program is ABI-equivalent with Solidity, which means you can call it from both Solidity and Rust.
//! To do this, run `cargo stylus export-abi`.
//!
//! Note: this code is a template-only and has not been audited.
//!
// Allow `cargo stylus export-abi` to generate a main function.
// #![cfg_attr(not(feature = "export-abi"), no_main)]
#![cfg_attr(not(any(test)), no_main)]
extern crate alloc;

use alloc::vec::Vec;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256},
    crypto::keccak,
    prelude::*,
};

// Lyubo: Reuse the macros from the cpu verifier
macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($msg.to_vec());
        }
    };
}

pub trait MemoryPageFactRegistryConstants {
    const REGULAR_PAGE: [u8; 32] = U256::from_limbs([0, 0, 0, 0]).to_be_bytes();
    const CONTINUOUS_PAGE: [u8; 32] = U256::from_limbs([1, 0, 0, 0]).to_be_bytes();
}

// Use sol_storage! macro instead of manual #[storage] attributes
sol_storage! {
    #[entrypoint]
    pub struct MemoryPageFactRegistry {
        #[borrow]
        FactRegistry fact_registry;
    }

    pub struct FactRegistry {
        mapping(bytes32 => bool) verified_fact;
        bool any_fact_registered;
    }
}

trait IQueryableFactRegistry {
    fn has_registered_fact(&self) -> bool;
    fn is_valid(&self, fact: FixedBytes<32>) -> bool;
}

// Combine trait implementations into a single #[public] block

impl FactRegistry {
    fn _fact_check(&self, fact: FixedBytes<32>) -> bool {
        self.verified_fact.get(fact)
    }

    fn register_fact(&mut self, fact: FixedBytes<32>) {
        self.verified_fact.setter(fact).set(true);
        if !self.any_fact_registered.get() {
            self.any_fact_registered.set(true);
        }
    }
}

#[public]
impl IQueryableFactRegistry for FactRegistry {
    fn has_registered_fact(&self) -> bool {
        self.any_fact_registered.get()
    }
    fn is_valid(&self, fact: FixedBytes<32>) -> bool {
        self._fact_check(fact)
    }
}

impl MemoryPageFactRegistryConstants for MemoryPageFactRegistry {}

#[public]
#[inherit(FactRegistry)]
impl MemoryPageFactRegistry {
    // This exposes is_valid and has_registered_fact via inheritance

    #[inline]
    pub fn register_regular_memory_page(
        &mut self,
        memory_pairs: Vec<U256>, // Keep Vec for ABI compatibility
        z: U256,
        alpha: U256,
        prime: U256,
    ) -> Result<(FixedBytes<32>, FixedBytes<32>, U256), Vec<u8>> {
        require!(
            memory_pairs.len() < 2usize.pow(20),
            b"Too many memory values."
        );
        require!(
            memory_pairs.len() % 2 == 0,
            b"Size of memoryPairs must be even."
        );
        require!(z < prime, b"Invalid value of z.");
        require!(alpha < prime, b"Invalid value of alpha.");

        let (fact_hash, memory_hash, prod) =
            Self::compute_fact_hash(&memory_pairs, z, alpha, prime);
        self.fact_registry.register_fact(fact_hash);

        Ok((fact_hash, memory_hash, prod))
    }
}

impl MemoryPageFactRegistry {
    fn compute_fact_hash(
        memory_pairs: &Vec<U256>,
        z: U256,
        alpha: U256,
        prime: U256,
    ) -> (FixedBytes<32>, FixedBytes<32>, U256) {
        let mut prod = U256::from(1);
        let mut memory_data = Vec::with_capacity(memory_pairs.len() * 32);

        for pair in memory_pairs.chunks(2) {
            let val_alpha = pair[1].mul_mod(alpha, prime);
            let address_value_lin_comb = pair[0].add_mod(val_alpha, prime);
            let term = z + prime - address_value_lin_comb;
            prod = prod.mul_mod(term, prime);

            memory_data.extend_from_slice(&pair[0].to_be_bytes::<32>());
            memory_data.extend_from_slice(&pair[1].to_be_bytes::<32>());
        }

        let memory_hash_output: FixedBytes<32> = keccak(&memory_data).into();

        let mut hash_buffer = Vec::with_capacity(256);
        hash_buffer.extend_from_slice(&Self::REGULAR_PAGE);
        hash_buffer.extend_from_slice(&prime.to_be_bytes::<32>());
        hash_buffer.extend_from_slice(&U256::from(memory_pairs.len() / 2).to_be_bytes::<32>());
        hash_buffer.extend_from_slice(&z.to_be_bytes::<32>());
        hash_buffer.extend_from_slice(&alpha.to_be_bytes::<32>());
        hash_buffer.extend_from_slice(&prod.to_be_bytes::<32>());
        hash_buffer.extend_from_slice(&memory_hash_output.as_slice());
        hash_buffer.extend_from_slice(&[0; 32]);
        let fact_hash_output: FixedBytes<32> = keccak(&hash_buffer).into();

        (fact_hash_output, memory_hash_output, prod)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{B256, U256};

    #[motsu::test]
    fn test_compute_fact_hash() {
        // Create memory pairs with the same values as in the Solidity test
        let mut memory_pairs: Vec<U256> = Vec::new();
        memory_pairs.push(U256::from(1));
        memory_pairs.push(U256::from(100));
        memory_pairs.push(U256::from(2));
        memory_pairs.push(U256::from(200));

        let z = U256::from(5);
        let alpha = U256::from(3);

        // Prime field used by StarkWare: 2^251 + 17 * 2^192 + 1
        // let prime = 2u128.pow(251) + 17 * 2u128.pow(192) + 1;
        let prime = {
            let mut p: U256 = U256::from(1) << 251;
            p += U256::from(17) << 192;
            p += U256::from(1);
            p
        };

        // Call the compute_fact_hash function
        let (fact_hash, memory_hash, prod) =
            MemoryPageFactRegistry::compute_fact_hash(&memory_pairs, z, alpha, prime);

        // Compare only the product which is a simple value
        assert_eq!(prod, U256::from(176712), "Product mismatch");
        let expected_memory_hash = U256::from_str_radix(
            "73303762061477191319875668523507331965327761895046903539761298990706739567530",
            10,
        )
        .unwrap();

        // For the complex hash values, just ensure they're not zero
        assert_eq!(
            U256::from_be_bytes(memory_hash.into()),
            expected_memory_hash,
            "Memory hash mismatch"
        );

        // For fact_hash comparison, use hex string
        let expected_fact_hash = B256::from_slice(
            &hex::decode("eb4573be19285f49cf74a74d3b35b14a8d601493ea9c2bf199eb34b4ebc0f5c7")
                .unwrap(),
        );
        assert_eq!(
            B256::from_slice(&fact_hash.as_slice()),
            expected_fact_hash,
            "Fact hash mismatch"
        );
    }
}
