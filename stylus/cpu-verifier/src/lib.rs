//! Stylus Cpu Verifier
//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[path = "prime-field-element0.rs"]
mod prime_field_element0;
#[path = "layout-specific.rs"]
mod layout_specific;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

// use macros::require;
use offsets::PublicMemoryOffset;
use layout_specific::LayoutSpecific;
use prime_field_element0::PrimeFieldElement0;
// mod stark_verifier;

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256},
    crypto::keccak,
};


// #[storage]
pub struct CpuVerifier {}

impl PublicMemoryOffset for CpuVerifier {
    fn get_public_memory_offset() -> usize {
        22
    }
}

// #[public]
impl CpuVerifier {
    // pub fn verify_proof_external(
    //     &self,
    //     proof_params: Vec<U256>,
    //     proof: Vec<U256>,
    //     public_input: Vec<U256>,
    // ) -> Result<(), stark_verifier::VerifierError> {
    //     // Lyubo: Should call self
    //     stark_verifier::StarkVerifier {}.verify_proof(&proof_params, &proof, &public_input)
    // }

    pub fn get_public_input_hash(&self, public_input: &[U256]) -> FixedBytes<32> {
        let n_pages = public_input[21].to::<usize>();
        let mut input_data = Vec::new();
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

    pub fn oods_consistency_check(ctx: &mut [U256], public_input: &[U256]) -> U256 {
        CpuVerifier::verify_memory_page_facts(ctx, public_input);
        ctx[331] = ctx[352];
        ctx[332] = ctx[353];
        ctx[334] = ctx[354];
        
        let public_memory_prod = CpuVerifier::compute_public_memory_quotient(ctx, public_input);
        ctx[333] = public_memory_prod;

        LayoutSpecific::prepare_for_oods_check(ctx)

        // let composition_from_trace_value = U256::from_be_slice(&static_call(
        //     Call::new_in(self), 
        //     oodsContractAddress,
        //     &ctx[1 + MM_CONSTRAINT_POLY_ARGS_START..MM_CONSTRAINT_POLY_ARGS_END - MM_CONSTRAINT_POLY_ARGS_START]
        // ));

        // let claimed_composition = fadd(ctx[MM_COMPOSITION_OODS_VALUES], fmul(ctx[MM_OODS_POINT], ctx[MM_COMPOSITION_OODS_VALUES + 1]));
        // require!(composition_from_trace_value == claimed_composition, b"claimedComposition does not match trace");
    }

    fn verify_memory_page_facts(ctx: &[U256], public_input: &[U256]) {
        let n_public_memory_pages = ctx[1276].to::<usize>();
        for page in 0..n_public_memory_pages {
            let memory_hash_ptr = ctx[5].to::<usize>() + CpuVerifier::get_offset_page_hash(page);
            let prod_ptr = ctx[5].to::<usize>() + CpuVerifier::get_offset_page_prod(page, n_public_memory_pages);
            let page_size_ptr = ctx[5].to::<usize>() + CpuVerifier::get_offset_page_size(page);

            let memory_hash = public_input[memory_hash_ptr];
            let prod = public_input[prod_ptr];
            let page_size = public_input[page_size_ptr];

            let mut pageAddr = U256::ZERO;
            if page > 0 {
                let page_addr_ptr = ctx[5].to::<usize>() + CpuVerifier::get_offset_page_addr(page);
                pageAddr = public_input[page_addr_ptr];
            }

            let mut page_type = U256::from(1);
            if page == 0 {
                page_type = U256::from(0);
            }
            let mut hash_buffer = Vec::new();
            hash_buffer.extend_from_slice(&page_type.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&PrimeFieldElement0::K_MODULUS.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&page_size.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&ctx[352].to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&ctx[353].to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&prod.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&memory_hash.to_be_bytes::<32>());
            hash_buffer.extend_from_slice(&pageAddr.to_be_bytes::<32>());
            let fact_hash_output: FixedBytes<32> = keccak(&hash_buffer).into();

            // Lyubo: Use memory_page_fact_registry from the storage
            // require!(memory_page_fact_registry.is_valid(fact_hash_output), b"Memory page fact was not registered.");
        }
    }

    fn compute_public_memory_quotient(ctx: &[U256], public_input: &[U256]) -> U256 {
        let n_values = ctx[1275];
        let z = ctx[331];
        let alpha = ctx[332];
        // Lyubo: Use safe_div from layout_specific.rs
        let public_memory_size = LayoutSpecific::safe_div(ctx[324], U256::from(16));
        // require!(n_values < uint!(16777216_U256), "Overflow protection failed.");
        // require!(n_values <= public_memory_size, "Number of values of public memory is too large.");

        let n_public_memory_pages = ctx[1276].to::<usize>();
        let cumulative_prods_ptr = ctx[5].to::<usize>() + CpuVerifier::get_offset_page_prod(0, n_public_memory_pages);
        let denominator = CpuVerifier::compute_public_memory_prod(public_input, cumulative_prods_ptr, n_public_memory_pages, PrimeFieldElement0::K_MODULUS);
        
        let padding_addr_ptr = ctx[5].to::<usize>() + 19;
        let padding_addr = public_input[padding_addr_ptr];
        let padding_value = public_input[padding_addr_ptr + 1];
        
        let hash_first_address_value = PrimeFieldElement0::fadd(padding_addr, PrimeFieldElement0::fmul(padding_value, alpha));
        let denom_pad = PrimeFieldElement0::fpow(PrimeFieldElement0::fsub(z, hash_first_address_value), public_memory_size - n_values);
        let denominator = PrimeFieldElement0::fmul(denominator, denom_pad);
        let numerator = PrimeFieldElement0::fpow(z, public_memory_size);
        let result = PrimeFieldElement0::fmul(numerator, PrimeFieldElement0::inverse(denominator));
        result
        // Ok(result)
    }

    fn compute_public_memory_prod(public_input: &[U256], cumulative_prods_ptr: usize, n_public_memory_pages: usize, prime: U256) -> U256 {
        let mut res = U256::from(1);
        let last_ptr = cumulative_prods_ptr + n_public_memory_pages;
        for ptr in cumulative_prods_ptr..last_ptr {
            res = res.mul_mod(public_input[ptr], prime);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod test_constants;
    use stylus_sdk::{
        alloy_primitives::{U256, uint},
    };

    #[motsu::test]
    fn test_oods_consistency_check() {
        let mut proof = test_constants::get_proof();
        let mut ctx = test_constants::get_ctx_oods_consistency_check();
        let public_input = test_constants::get_public_input();
        let public_memory_prod = CpuVerifier::oods_consistency_check(&mut ctx, &public_input);
        assert_eq!(public_memory_prod, uint!(1552215061468209516830163195514878071221879601444981698864155012436627340325_U256));
    }

}
