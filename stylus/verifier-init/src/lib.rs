#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256, uint, Address},
    crypto::keccak,
    prelude::*,
};

use macros::require;

#[path = "prime-field-element0.rs"]
pub mod prime_field_element0;
#[path = "public-memory-offset.rs"]
pub mod public_memory_offset;
use crate::public_memory_offset::PublicMemoryOffset;
use crate::prime_field_element0::PrimeFieldElement0;

#[storage]
#[entrypoint]
pub struct VerifierInit;

#[public]
impl VerifierInit {

    #[inline]
    fn init_verifier_params(
        &mut self,
        public_input: Vec<U256>,
        proof_params: Vec<U256>,
    ) -> Result<(Vec<U256>, Vec<U256>), Vec<u8>> {
        require!(
            proof_params.len() >= 5,
            "Invalid proof params"
        );
       
        require!(
            proof_params.len() == (5 + proof_params[4].to::<usize>()),
            "Invalid proofParams."
        );
        let log_blowup_factor = proof_params[1];
        require!(log_blowup_factor <= U256::from(16), "logBlowupFactor must be at most 16");
        require!(log_blowup_factor >= U256::from(1), "logBlowupFactor must be at least 1");
        
        let proof_of_work_bits = proof_params[2];
        require!(proof_of_work_bits <= U256::from(50), "proofOfWorkBits must be at most 50");
        require!(proof_of_work_bits >= U256::from(1), "minimum proofOfWorkBits not satisfied");
        // require!(proof_of_work_bits < num_security_bits, "Proofs may not be purely based on PoW.");

        let log_fri_last_layer_deg_bound = proof_params[3];
        require!(log_fri_last_layer_deg_bound <= U256::from(10), "logFriLastLayerDegBound must be at most 10.");

        let n_fri_steps = proof_params[4].to::<usize>();
        require!(n_fri_steps <= 10, "Too many fri steps.");
        require!(n_fri_steps > 1, "Not enough fri steps.");

        let mut fri_step_sizes: Vec<U256> = Vec::new();
        for i in 0..n_fri_steps {
            fri_step_sizes.push(proof_params[5 + i]);
        }

        let (mut ctx, log_trace_length) = Self::air_specific_init(&public_input)?;
        Self::validate_fri_params(&fri_step_sizes, log_trace_length, log_fri_last_layer_deg_bound)?;

        ctx[315] = U256::from(1) << log_fri_last_layer_deg_bound;
        ctx[324] = U256::from(1) << log_trace_length;
        ctx[1] = U256::from(1) << log_blowup_factor;
        ctx[3] = proof_of_work_bits;

        let n_queries = proof_params[0];
        require!(n_queries > U256::ZERO, "Number of queries must be at least one");
        require!(n_queries <= U256::from(48), "Too many queries.");
        // require!(
        //     n_queries * log_blowup_factor + proof_of_work_bits >= num_security_bits,
        //     "Proof params do not satisfy security requirements."
        // );

        ctx[9] = n_queries;
        ctx[2] = log_trace_length + log_blowup_factor;
        ctx[0] = U256::from(1) << ctx[2];

        let gen_eval_domain = PrimeFieldElement0::fpow(U256::from(3), (PrimeFieldElement0::K_MODULUS - U256::from(1)) / ctx[0]);
        ctx[4] = gen_eval_domain;
        ctx[350] = PrimeFieldElement0::fpow(gen_eval_domain, ctx[1]);

        Ok((ctx, fri_step_sizes))
    }
}

impl VerifierInit {

    fn validate_fri_params(
        fri_step_sizes: &[U256],
        log_trace_length: U256,
        log_fri_last_layer_deg_bound: U256,
    ) -> Result<(), Vec<u8>> {
        require!(fri_step_sizes[0] == U256::ZERO, "Only eta0 == 0 is currently supported");

        let mut expected_log_deg_bound = log_fri_last_layer_deg_bound;
        let n_fri_steps = fri_step_sizes.len();
        for i in 1..n_fri_steps {
            let fri_step_size = fri_step_sizes[i];
            require!(fri_step_size >= U256::from(2), "Min supported fri step size is 2.");
            require!(fri_step_size <= U256::from(4), "Max supported fri step size is 4.");
            expected_log_deg_bound += fri_step_size;
        }

        require!(expected_log_deg_bound == log_trace_length, "Fri params do not match trace length");
        Ok(())
    }

    fn air_specific_init(public_input: &[U256]) -> Result<(Vec<U256>, U256), Vec<u8>> {
        require!(public_input.len() >= 22, "publicInput is too short.");
        let mut ctx = vec![U256::ZERO; 1277];
        ctx[325] = U256::from(65536);
        ctx[326] = U256::from(32768);

        let log_n_steps = public_input[1];
        require!(log_n_steps < U256::from(50), "Number of steps is too large.");
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

        Self::layout_specific_init(&mut ctx, public_input)?;

        Ok((ctx, log_trace_length))
    }

    fn layout_specific_init(ctx: &mut [U256], public_input: &[U256]) -> Result<(), Vec<u8>> {
        let output_begin_addr = public_input[9];
        let output_stop_ptr = public_input[10];
        require!(output_begin_addr <= output_stop_ptr, "output begin_addr must be <= stop_ptr");
        require!(output_stop_ptr < uint!(18446744073709551616_U256), "Out of range output stop_ptr.");

        let n_steps = U256::from(1) << ctx[1274];
        ctx[346] = public_input[11];
        Self::validate_builtin_pointers(ctx[346], public_input[12], U256::from(128), U256::from(3), n_steps)?;

        ctx[344] = uint!(2089986280348253421170679821480865132823066470938446095505822317253594081284_U256);
        ctx[345] = uint!(1713931329540660377023406109199410414810705867260802078187082345529207694986_U256);
        ctx[347] = public_input[13];
        Self::validate_builtin_pointers(ctx[347], public_input[14], U256::from(8), U256::from(1), n_steps)?;

        ctx[335] = U256::from(1);
        ctx[348] = public_input[15];
        Self::validate_builtin_pointers(ctx[348], public_input[16], U256::from(8), U256::from(5), n_steps)?;

        ctx[339] = U256::from(1);
        ctx[340] = U256::from(0);

        ctx[349] = public_input[17];
        Self::validate_builtin_pointers(ctx[349], public_input[18], U256::from(8), U256::from(6), n_steps)?;

        Ok(())
    }

    fn validate_builtin_pointers(
        initial_address: U256,
        stop_address: U256,
        builtin_ratio: U256,
        cells_per_instance: U256,
        n_steps: U256
    ) -> Result<(), Vec<u8>> {
        require!(
            initial_address < uint!(18446744073709551616_U256),
            "Out of range begin_addr."
        );
        let max_stop_ptr = initial_address + cells_per_instance * Self::safe_div(n_steps, builtin_ratio)?;
        require!(
            initial_address <= stop_address && stop_address <= max_stop_ptr,
            "Invalid stop_ptr."
        );
        Ok(())
    }

    fn safe_div(
        numerator: U256,
        denominator: U256
    ) -> Result<U256, Vec<u8>> {
        require!(
            denominator != U256::ZERO,
            "The denominator must not be zero"
        );
        require!(
            numerator % denominator == U256::ZERO,
            "The numerator is not divisible by the denominator."
        );
        Ok(numerator / denominator)
    }

}