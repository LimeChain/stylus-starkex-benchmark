//! Stylus Cpu Verifier
//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
// #![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[path = "stark-verifier.rs"]
pub mod stark_verifier;
#[path = "layout-specific.rs"]
pub mod layout_specific;
#[path = "verifier-channel.rs"]
pub mod verifier_channel;
#[path = "prime-field-element0.rs"]
pub mod prime_field_element0;
#[path = "public-memory-offset.rs"]
pub mod public_memory_offset;
#[path = "fri-statement-verifier.rs"]
pub mod fri_statement_verifier;
#[path = "merkle-statement-verifier.rs"]
pub mod merkle_statement_verifier;

#[cfg(test)]
#[path = "tests/test_constants.rs"]
pub mod test_constants;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use macros::require;

use crate::stark_verifier::StarkVerifier;
use crate::layout_specific::LayoutSpecific;
use crate::public_memory_offset::PublicMemoryOffset;
use crate::prime_field_element0::PrimeFieldElement0;
use crate::merkle_statement_verifier::MerkleStatementVerifier;

use stylus_sdk::{
    alloy_primitives::{address, FixedBytes, U256, uint},
    crypto::keccak,
    call::{static_call, Call},
    prelude::*,
};


// #[storage]
pub struct CpuVerifier {}

#[public]
#[inherit(StarkVerifier)]
impl CpuVerifier {
    
    fn air_specific_init(public_input: &[U256]) -> Result<(Vec<U256>, U256), Vec<u8>> {
        // require!(public_input.len() >= 22, "publicInput is too short.");
        let mut ctx = vec![U256::ZERO; 1277];
        ctx[325] = U256::from(65536);
        ctx[326] = U256::from(32768);

        let log_n_steps = public_input[1];
        // require!(log_n_steps < U256::from(50), "Number of steps is too large.");
        ctx[1274] = log_n_steps;
        let log_trace_length = log_n_steps + U256::from(4);
        
        ctx[336] = public_input[2];
        ctx[337] = public_input[3];
        require!(ctx[336] <= ctx[337], "rc_min must be <= rc_max");
        require!(ctx[337] < ctx[325], "rc_max out of range");
        require!(public_input[4] == uint!(42800643258479064999893963318903811951182475189843316_U256), "Layout code mismatch.");

        ctx[328] = public_input[5];
        ctx[330] = public_input[6];
        require!(ctx[328] == U256::from(1), "Invalid initial pc");
        require!(ctx[330] == U256::from(5), "Invalid final pc");

        ctx[327] = public_input[7];
        ctx[329] = public_input[8];
        require!(public_input[21] >= U256::from(1) && public_input[21] < U256::from(100000), "Invalid number of memory pages.");

        ctx[1276] = public_input[21];

        let mut n_public_memory_entries = U256::from(0);
        for page in 0..ctx[1276].to::<usize>() {
            let n_page_entries = public_input[PublicMemoryOffset::get_offset_page_size(page)];
            require!(n_page_entries < U256::from(1073741824), "Too many public memory entries in one page.");
            n_public_memory_entries += n_page_entries;
        }
        ctx[1275] = n_public_memory_entries;

        let expected_public_input_length = PublicMemoryOffset::get_public_input_length(ctx[1276].to::<usize>());
        require!(expected_public_input_length == public_input.len(), "Public input length mismatch.");

        LayoutSpecific::layout_specific_init(&mut ctx, public_input)?;

        Ok((ctx, log_trace_length))
    }

    fn oods_consistency_check(&self, ctx: &mut [U256], public_input: &[U256]) -> Result<(), Vec<u8>> {
        CpuVerifier::verify_memory_page_facts(ctx, public_input);
        ctx[331] = ctx[352];
        ctx[332] = ctx[353];
        ctx[334] = ctx[354];
        
        let public_memory_prod = CpuVerifier::compute_public_memory_quotient(ctx, public_input)?;
        ctx[333] = public_memory_prod;

        LayoutSpecific::prepare_for_oods_check(ctx)?;

        // let composition_from_trace_value = U256::from_be_slice(&static_call(
        //     Call::new_in(self), 
        //     oodsContractAddress,
        //     &ctx[318..551]
        // ));

        // let claimed_composition = PrimeFieldElement0::fadd(ctx[551], PrimeFieldElement0::fmul(ctx[351], ctx[552]));
        // require!(composition_from_trace_value == claimed_composition, b"claimedComposition does not match trace");
        Ok(())
    }

    fn get_public_input_hash(public_input: &[U256]) -> FixedBytes<32> {
        let n_pages = public_input[21].to::<usize>();
        let mut input_data = Vec::new();
        for i in 1..n_pages {
            input_data.extend_from_slice(&public_input[i].to_be_bytes::<32>());
        }

        keccak(&input_data).into()
    }
}

impl CpuVerifier {

    fn verify_memory_page_facts(ctx: &[U256], public_input: &[U256]) {
        let n_public_memory_pages = ctx[1276].to::<usize>();
        for page in 0..n_public_memory_pages {
            let memory_hash_ptr = ctx[5].to::<usize>() + PublicMemoryOffset::get_offset_page_hash(page);
            let prod_ptr = ctx[5].to::<usize>() + PublicMemoryOffset::get_offset_page_prod(page, n_public_memory_pages);
            let page_size_ptr = ctx[5].to::<usize>() + PublicMemoryOffset::get_offset_page_size(page);

            let memory_hash = public_input[memory_hash_ptr];
            let prod = public_input[prod_ptr];
            let page_size = public_input[page_size_ptr];

            let mut page_addr = U256::ZERO;
            if page > 0 {
                let page_addr_ptr = ctx[5].to::<usize>() + PublicMemoryOffset::get_offset_page_addr(page);
                page_addr = public_input[page_addr_ptr];
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
            hash_buffer.extend_from_slice(&page_addr.to_be_bytes::<32>());
            // let fact_hash_output: FixedBytes<32> = keccak(&hash_buffer).into();

            // Lyubo: Use memory_page_fact_registry from the storage
            // require!(memory_page_fact_registry.is_valid(fact_hash_output), b"Memory page fact was not registered.");
        }
    }

    fn compute_public_memory_quotient(ctx: &[U256], public_input: &[U256]) -> Result<U256, Vec<u8>> {
        let n_values = ctx[1275];
        let z = ctx[331];
        let alpha = ctx[332];
        
        let public_memory_size = LayoutSpecific::safe_div(ctx[324], U256::from(16))?;
        require!(n_values < uint!(16777216_U256), "Overflow protection failed.");
        require!(n_values <= public_memory_size, "Number of values of public memory is too large.");

        let n_public_memory_pages = ctx[1276].to::<usize>();
        let cumulative_prods_ptr = ctx[5].to::<usize>() + PublicMemoryOffset::get_offset_page_prod(0, n_public_memory_pages);
        let denominator = CpuVerifier::compute_public_memory_prod(public_input, cumulative_prods_ptr, n_public_memory_pages, PrimeFieldElement0::K_MODULUS);
        
        let padding_addr_ptr = ctx[5].to::<usize>() + 19;
        let padding_addr = public_input[padding_addr_ptr];
        let padding_value = public_input[padding_addr_ptr + 1];
        
        let hash_first_address_value = PrimeFieldElement0::fadd(padding_addr, PrimeFieldElement0::fmul(padding_value, alpha));
        let denom_pad = PrimeFieldElement0::fpow(PrimeFieldElement0::fsub(z, hash_first_address_value), public_memory_size - n_values);
        let denominator = PrimeFieldElement0::fmul(denominator, denom_pad);
        let numerator = PrimeFieldElement0::fpow(z, public_memory_size);
        let result = PrimeFieldElement0::fmul(numerator, PrimeFieldElement0::inverse(denominator));
        Ok(result)
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
    use crate::test_constants;
    use test_utils::try_execute;
    use stylus_sdk::{
        alloy_primitives::{U256, uint},
    };

    #[motsu::test]
    fn test_oods_consistency_check() {
        // let mut proof = test_constants::get_proof();
        // let mut ctx = test_constants::get_ctx_oods_consistency_check();
        // let public_input = test_constants::get_public_input();
        // Self::oods_consistency_check(&mut ctx, &public_input);
    }

    #[motsu::test]
    fn test_air_specific_init() {
        let public_input = test_constants::get_public_input();
        let (ctx, log_trace_length) = try_execute!(CpuVerifier::air_specific_init(&public_input));
        let ctx_expected = test_constants::get_ctx_air_specific_init();

        for i in 0..ctx.len() {
            assert_eq!(ctx[i], ctx_expected[i]);
        }
        assert_eq!(log_trace_length, U256::from(26));
    }

    #[motsu::test]
    fn test_init_verifier_params() {
        let public_input = test_constants::get_public_input();
        let proof_params = test_constants::get_proof_params();
        let (ctx, _) = try_execute!(CpuVerifier::init_verifier_params(&public_input, &proof_params));
        let ctx_expected = test_constants::get_ctx_init_verifier_params();
        for i in 0..ctx.len() {
            assert_eq!(ctx[i], ctx_expected[i]);
        }
    }

    #[motsu::test]
    fn test_read_last_fri_layer() {
        let mut proof = test_constants::get_proof();
        let mut ctx = test_constants::get_ctx_read_last_fri_layer();
        try_execute!(CpuVerifier::read_last_fri_layer(&mut proof, &mut ctx));

        assert_eq!(ctx[10], uint!(268_U256));
        assert_eq!(ctx[11], uint!(101063039785234930674416911940782140361807536835453250352760633033315826439229_U256));
        assert_eq!(ctx[316], uint!(204_U256));
    }

    // // Lyubo: Should fix this test
    // // #[motsu::test]
    // // fn test_compute_first_fri_layer() {
    // //     let mut proof = test_constants::get_proof();
    // //     let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
    // //     CpuVerifier::compute_first_fri_layer(&mut proof, &mut ctx);
    // // }

    #[motsu::test]
    fn test_adjust_query_indices_and_prepare_eval_points() {
        let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
        CpuVerifier::adjust_query_indices_and_prepare_eval_points(&mut ctx);
        assert_eq!(ctx[553], uint!(3515892385904170702434114719646176958489529091479346127319408828731691841909_U256));
        assert_eq!(ctx[109], uint!(4818245268_U256));
        assert_eq!(ctx[139], uint!(8285752452_U256));
    }

    // Lyubo: Finish this test with asserts
    #[motsu::test]
    fn test_read_query_responses_and_decommit() {
        let mut proof = test_constants::get_proof();
        let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
        CpuVerifier::adjust_query_indices_and_prepare_eval_points(&mut ctx);

        ctx[10] = U256::from(8584); // proof pointer
        let merkle_root = CpuVerifier::u256_to_bytes(ctx[6]);
        try_execute!(CpuVerifier::read_query_responses_and_decommit(&mut proof, &mut ctx, 12, 9, 602, merkle_root));
    }

}
