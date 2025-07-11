//! Stylus Cpu Verifier
//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;
mod consts;
mod layout_specific;
mod macros;
mod public_memory_offsets;
mod stark_verifier;

use offsets::{public_input_offsets, PublicMemoryOffset};

// use stark_verifier::*;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, prelude::*};

// #[storage]
// pub struct LayoutSpecific {}

#[storage]
pub struct CpuVerifier {}

impl PublicMemoryOffset for CpuVerifier {
    fn get_public_memory_offset() -> usize {
        public_input_offsets::OFFSET_PUBLIC_MEMORY
    }
}

#[public]
impl CpuVerifier {
    pub fn verify_proof_external(
        &self,
        proof_params: Vec<U256>,
        proof: Vec<U256>,
        public_input: Vec<U256>,
    ) -> Result<(), stark_verifier::VerifierError> {
        stark_verifier::StarkVerifier {}.verify_proof(&proof_params, &proof, &public_input)
    }

    pub fn get_public_input_hash(&self, public_input: &[U256]) -> FixedBytes<32> {
        let n_pages = public_input[OFFSET_N_PUBLIC_MEMORY_PAGES];
        let mut input_data = Vec::with_capacity(n_pages * 32);
        for i in 1..n_pages {
            input_data.extend_from_slice(&public_input[i].to_be_bytes::<32>());
        }

        keccak(&input_data).into()
    }

    pub fn get_n_interaction_elements(&self) -> usize {
        6
    }

    pub fn get_mm_interaction_elements(&self) -> usize {
        352
    }

    pub fn get_mm_oods_values(&self) -> usize {
        359
    }

    pub fn get_n_oods_values(&self) -> usize {
        194
    }

    pub fn oods_consistency_check(ctx: &[U256]) {
        verify_memory_page_facts(ctx);
        ctx[MM_MEMORY__MULTI_COLUMN_PERM__PERM__INTERACTION_ELM] = ctx[MM_INTERACTION_ELEMENTS];
        ctx[MM_MEMORY__MULTI_COLUMN_PERM__HASH_INTERACTION_ELM0] = ctx[MM_INTERACTION_ELEMENTS + 1];
        ctx[MM_RANGE_CHECK16__PERM__INTERACTION_ELM] = ctx[MM_INTERACTION_ELEMENTS + 2];
        
        let public_memory_prod = compute_public_memory_quotient(ctx);
        ctx[MM_MEMORY__MULTI_COLUMN_PERM__PERM__PUBLIC_MEMORY_PROD] = public_memory_prod;

        prepare_for_oods_check(ctx);

        let composition_from_trace_value = U256::from_be_slice(&static_call(
            Call::new_in(self), 
            oodsContractAddress,
            &ctx[1 + MM_CONSTRAINT_POLY_ARGS_START..MM_CONSTRAINT_POLY_ARGS_END - MM_CONSTRAINT_POLY_ARGS_START]
        ));

        let claimed_composition = fadd(ctx[MM_COMPOSITION_OODS_VALUES], fmul(ctx[MM_OODS_POINT], ctx[MM_COMPOSITION_OODS_VALUES + 1]));
        require!(composition_from_trace_value == claimed_composition, b"claimedComposition does not match trace");
    }

    fn verify_memory_page_facts(ctx: &[U256]) {
        let n_public_memory_pages = ctx[MM_N_PUBLIC_MEM_PAGES];

        for page in 0..n_public_memory_pages {
            // Lyubo: Use get_offset_page_hash, get_offset_page_prod, get_offset_page_size from public_memory_offsets.rs
            let memory_hash_ptr = ctx[MM_PUBLIC_INPUT_PTR] + get_offset_page_hash(page);
            let prod_ptr = ctx[MM_PUBLIC_INPUT_PTR] + get_offset_page_prod(page, n_public_memory_pages);
            let page_size_ptr = ctx[MM_PUBLIC_INPUT_PTR] + get_offset_page_size(page);

            let memory_hash = ctx[memory_hash_ptr];
            let prod = ctx[prod_ptr];
            let page_size = ctx[page_size_ptr];

            let pageAddr = U256::ZERO;
            if page > 0 {
                // Lyubo: Use get_offset_page_addr from public_memory_offsets.rs
                let page_addr_ptr = ctx[MM_PUBLIC_INPUT_PTR] + get_offset_page_addr(page);
                pageAddr = ctx[page_addr_ptr];
            }

            let mut hash_buffer = Vec::with_capacity(256);
            // page == 0 ? REGULAR_PAGE : CONTINUOUS_PAGE,
            hash_buffer.extend_from_slice(page == 0 ? &REGULAR_PAGE.to_be_bytes::<32>() : &CONTINUOUS_PAGE.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&K_MODULUS.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&page_size.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&ctx[MM_INTERACTION_ELEMENTS].to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&ctx[MM_INTERACTION_ELEMENTS + 1].to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&prod.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&memory_hash.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&pageAddr.to_be_bytes::<32>());
            let fact_hash_output: FixedBytes<32> = keccak(&hash_buffer).into();
            // Lyubo: Use memory_page_fact_registry from the storage
            require!(memory_page_fact_registry.is_valid(fact_hash_output), b"Memory page fact was not registered.");
        }
    }

    fn compute_public_memory_quotient(ctx: &[U256]) -> U256 {
        let n_values = ctx[MM_N_PUBLIC_MEM_ENTRIES];
        let z = ctx[MM_MEMORY__MULTI_COLUMN_PERM__PERM__INTERACTION_ELM];
        let alpha = ctx[MM_MEMORY__MULTI_COLUMN_PERM__HASH_INTERACTION_ELM0];
        // Lyubo: Use safe_div from layout_specific.rs
        let public_memory_size = safe_div(ctx[MM_TRACE_LENGTH], PUBLIC_MEMORY_STEP);

        require!(n_values < 16777216, b"Overflow protection failed.");
        require!(n_values <= public_memory_size, b"Number of values of public memory is too large.");

        let n_public_memory_pages = ctx[MM_N_PUBLIC_MEM_PAGES];
        // Lyubo: Use get_offset_page_prod from public_memory_offsets.rs
        let cumulative_prods_ptr = ctx[MM_PUBLIC_INPUT_PTR] + get_offset_page_prod(0, n_public_memory_pages);
        let denominator = compute_public_memory_prod(cumulative_prods_ptr, n_public_memory_pages, K_MODULUS);

        let public_input_ptr = ctx[MM_PUBLIC_INPUT_PTR];
        let paddingAddrPtr = public_input_ptr + OFFSET_PUBLIC_MEMORY_PADDING_ADDR;
        let paddingAddr = ctx[paddingAddrPtr];
        let paddingValue = ctx[paddingAddrPtr + 1];

        let hash_first_address_value = fadd(paddingAddr, fmul(paddingValue, alpha));
        let denom_pad = fpow(fsub(z, hash_first_address_value), public_memory_size - n_values);
        let denominator = fmul(denominator, denom_pad);

        let numerator = fpow(z, public_memory_size);
        let result = fmul(numerator, inverse(denominator));
        result
    }

    fn compute_public_memory_prod(cumulative_prods_ptr: U256, n_public_memory_pages: U256, prime: U256) -> U256 {
        let mut res = U256::from(1);
        let last_ptr = cumulative_prods_ptr + n_public_memory_pages;
        for ptr in cumulative_prods_ptr..last_ptr {
            res = res.mul_mod(ctx[ptr], prime);
        }
        res
    }
}

pub const OUTPUT_BUILTIN_BIT: usize = 0;
pub const PEDERSEN_BUILTIN_BIT: usize = 1;
pub const RANGE_CHECK_BUILTIN_BIT: usize = 2;
pub const ECDSA_BUILTIN_BIT: usize = 3;
pub const BITWISE_BUILTIN_BIT: usize = 4;
pub const EC_OP_BUILTIN_BIT: usize = 5;
pub const KECCAK_BUILTIN_BIT: usize = 6;
pub const POSEIDON_BUILTIN_BIT: usize = 7;
