//!
//! The program is ABI-equivalent with Solidity, which means you can call it from both Solidity and Rust.
//! To do this, run `cargo stylus export-abi`.
//!
//! Note: this code is a template-only and has not been audited.
//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test)), no_main)]
extern crate alloc;

use std::io::Read;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::keccak256,
    alloy_primitives::{B256, U256},
    console,
    evm::{gas_left, ink_left},
    prelude::*,
};

macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($msg.to_vec());
        }
    };
}

pub trait MemoryPageFactRegistryConstants {
    const REGULAR_PAGE: U256 = U256::from_limbs([0, 0, 0, 0]);
    const CONTINUOUS_PAGE: U256 = U256::from_limbs([1, 0, 0, 0]);
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
    fn is_valid(&self, fact: B256) -> bool;
}

// Combine trait implementations into a single #[public] block

impl FactRegistry {
    fn _fact_check(&self, fact: B256) -> bool {
        self.verified_fact.get(fact)
    }

    fn register_fact(&mut self, fact: B256) {
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
    fn is_valid(&self, fact: B256) -> bool {
        self._fact_check(fact)
    }
}

impl MemoryPageFactRegistryConstants for MemoryPageFactRegistry {}

#[public]
#[inherit(FactRegistry)]
impl MemoryPageFactRegistry {
    // This exposes is_valid and has_registered_fact via inheritance

    pub fn register_regular_memory_page(
        &mut self,
        memory_pairs: Vec<U256>,
        z: U256,
        alpha: U256,
        prime: U256,
    ) -> Result<(B256, U256, U256), Vec<u8>> {
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
        let gas_left_before = self.vm().evm_gas_left();
        // console!("0.1. gas_left: {}", gas_left_before);
        let (fact_hash, memory_hash, prod) = Self::compute_fact_hash(memory_pairs, z, alpha, prime);
        self.fact_registry.register_fact(fact_hash);

        // console!(
        //     "0.2. spent gas by compute_fact_hash: {}",
        //     gas_left_before - self.vm().evm_gas_left()
        // );
        Ok((fact_hash, memory_hash, prod))
    }

    pub fn register_continuous_memory_page(
        &mut self,
        memory_pairs: Vec<U256>,
        z: U256,
        alpha: U256,
        prime: U256,
    ) -> Result<(B256, U256, U256), Vec<u8>> {
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

        let (fact_hash, memory_hash, prod) = Self::compute_fact_hash(memory_pairs, z, alpha, prime);
        self.fact_registry.register_fact(fact_hash);

        Ok((fact_hash, memory_hash, prod))
    }
}

// Private methods
// https://stylus-by-example.org/basic_examples/hashing
impl MemoryPageFactRegistry {
    fn compute_fact_hash(
        memory_pairs: Vec<U256>,
        z: U256,
        alpha: U256,
        prime: U256,
    ) -> (B256, U256, U256) {
        // console!("1. hello there! ink_left: {}", ink_left());
        let memory_size = memory_pairs.len() / 2;
        // console!("2. hello there! ink_left: {}", ink_left());
        let mut prod: alloy_primitives::Uint<256, 4> = U256::from(1);
        // console!("3. hello there! ink_left: {}", ink_left());
        for pair in memory_pairs.chunks_exact(2) {
            prod = prod.mul_mod(
                (z + prime - (pair[0] + pair[1].mul_mod(alpha, prime)) % prime) % prime,
                prime,
            );
        }
        console!("4. hello there! ink_left: {}", ink_left());
        use tiny_keccak::{Hasher, Keccak};
        let mut hasher: Keccak = Keccak::v256();
        for word in memory_pairs {
            hasher.update(&word.to_be_bytes::<32>());
        }
        let mut memory_hash_output = [0u8; 32];
        hasher.finalize(&mut memory_hash_output);
        console!("5. hello there! ink_left: {}", ink_left());
        // let memory_hash =
        //     U256::from_be_bytes::<32>(memory_hash_output.as_slice().try_into().unwrap());
        // let memory_hash_b256 = keccak256(&raw);
        // let memory_hash = U256::from_be_bytes(memory_hash_b256.0);

        // ── keccak( REGULAR_PAGE‖prime‖…‖0 ) ────────────────────────────────
        let mut hasher: Keccak = Keccak::v256();
        hasher.update(&Self::REGULAR_PAGE.to_be_bytes::<32>());
        hasher.update(&prime.to_be_bytes::<32>());
        hasher.update(&U256::from(memory_size).to_be_bytes::<32>());
        hasher.update(&z.to_be_bytes::<32>());
        hasher.update(&alpha.to_be_bytes::<32>());
        hasher.update(&prod.to_be_bytes::<32>());
        hasher.update(&memory_hash_output);
        hasher.update(&U256::ZERO.to_be_bytes::<32>());
        // let mut packed: Vec<u8> = Vec::with_capacity(8 * 32);
        // packed.extend_from_slice(&Self::REGULAR_PAGE.to_be_bytes::<32>());
        // packed.extend_from_slice(&prime.to_be_bytes::<32>());
        // packed.extend_from_slice(&U256::from(memory_size).to_be_bytes::<32>());
        // packed.extend_from_slice(&z.to_be_bytes::<32>());
        // packed.extend_from_slice(&alpha.to_be_bytes::<32>());
        // packed.extend_from_slice(&prod.to_be_bytes::<32>());
        // packed.extend_from_slice(&memory_hash_output);
        // packed.extend_from_slice(&U256::ZERO.to_be_bytes::<32>());

        // let fact_hash = keccak256(&packed);
        let mut fact_hash_output: [u8; 32] = [0u8; 32];
        hasher.finalize(&mut fact_hash_output);
        let fact_hash = B256::from_slice(&fact_hash_output);

        (fact_hash, U256::from_be_bytes(memory_hash_output), prod)
    }
}

// https://github.com/OffchainLabs/stylus-by-example/blob/master/src/app/basic_examples/mapping/page.mdx

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{B256, U256};

    #[motsu::test]
    fn test_compute_fact_hash() {
        // Create memory pairs with the same values as in the Solidity test
        let mut memory_pairs: Vec<alloy_primitives::Uint<256, 4>> = Vec::new();
        memory_pairs.push(U256::from(1));
        memory_pairs.push(U256::from(100));
        memory_pairs.push(U256::from(2));
        memory_pairs.push(U256::from(200));

        let z = U256::from(5);
        let alpha = U256::from(3);

        // Prime field used by StarkWare: 2^251 + 17 * 2^192 + 1
        let prime = {
            let mut p = U256::from(1) << 251;
            p += U256::from(17) << 192;
            p += U256::from(1);
            p
        };

        // Call the compute_fact_hash function
        let (fact_hash, memory_hash, prod) =
            MemoryPageFactRegistry::compute_fact_hash(memory_pairs, z, alpha, prime);

        // Just print the values for inspection instead of comparing
        println!("memory_hash: {:?}", memory_hash);
        println!("prod: {:?}", prod);
        println!("fact_hash: {:?}", fact_hash);

        // Compare only the product which is a simple value
        assert_eq!(prod, U256::from(176712), "Product mismatch");
        let expected_memory_hash = U256::from_str_radix(
            "73303762061477191319875668523507331965327761895046903539761298990706739567530",
            10,
        )
        .unwrap();
        // For the complex hash values, just ensure they're not zero
        assert_eq!(memory_hash, expected_memory_hash, "Memory hash mismatch");

        // For fact_hash comparison, use hex string
        let expected_fact_hash = B256::from_slice(
            &hex::decode("eb4573be19285f49cf74a74d3b35b14a8d601493ea9c2bf199eb34b4ebc0f5c7")
                .unwrap(),
        );
        assert_eq!(fact_hash, expected_fact_hash, "Fact hash mismatch");
    }
}
