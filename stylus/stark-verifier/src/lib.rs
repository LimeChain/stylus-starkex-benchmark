// use crate::consts::{input_offsets::*, memory_offsets::*, stark_parameters::*};
use macros::require;
// use stylus_sdk::alloy_primitives::{uint, U256};
// use stylus_sdk::alloy_sol_types::sol;
// use stylus_sdk::prelude::SolidityError;
// use stylus_sdk::call::{static_call, Call};

use stylus_sdk::{
    alloy_primitives::{uint, U256},
    crypto::keccak,
};

// use crate::public_memory_offsets::{offset_page_size, public_input_length};

// // TODO: switch to alloy::sol!
// sol! {
//     error InvalidProof(string reason);
//     error ComputationFailed(string reason);
// }

// #[derive(SolidityError)]
// pub enum VerifierError {
//     InvalidProof(InvalidProof),
//     ComputationFailed(ComputationFailed),
// }

// // hardcoded values from the 0x42AF9498647Be47A256C9cc8278eE94473Cb7771 contract constructor
// const MIN_POW_BITS: usize = 0;
// const NUM_SECURITY_BITS: usize = 80;

const PUBLIC_MP_LIMIT: U256 = uint!(1073741824_U256);

// const PROOF_PARAMS_N_QUERIES_OFFSET: usize = 0;
const PROOF_PARAMS_LOG_BLOWUP_FACTOR_OFFSET: usize = 1;
const PROOF_PARAMS_PROOF_OF_WORK_BITS_OFFSET: usize = 2;
const PROOF_PARAMS_FRI_LAST_LAYER_LOG_DEG_BOUND_OFFSET: usize = 3;
const PROOF_PARAMS_N_FRI_STEPS_OFFSET: usize = 4;
const PROOF_PARAMS_FRI_STEPS_OFFSET: usize = 5;

const INITIAL_PC: U256 = uint!(1_U256);
// FINAL_PC = INITIAL_PC + 4;
const FINAL_PC: U256 = uint!(5_U256);

// Stark parameters
// Air specific constants
const LOG_CPU_COMPONENT_HEIGHT: usize = 4;
const LAYOUT_CODE: U256 = uint!(42800643258479064999893963318903811951182475189843316_U256);
const PRIME_MINUS_ONE: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000000_U256);

pub struct StarkVerifier {}

impl StarkVerifier {
    // Algebraic Intermediate Representation (AIR) specific initialization.
    // fn air_specific_init(
    //     &self,
    //     public_input: &[U256],
    // ) -> Result<(Vec<U256>, usize), VerifierError> {
    //     require!(
    //         public_input.len() >= OFFSET_PUBLIC_MEMORY,
    //         "publicInput is too short."
    //     );
    //     let mut ctx: Vec<U256> = vec![U256::ZERO; MM_CONTEXT_SIZE];

    //     ctx[MM_OFFSET_SIZE] = U256::from(1u64 << 16);
    //     ctx[MM_HALF_OFFSET_SIZE] = U256::from(1u64 << 15);

    //     // Number of steps.
    //     let log_n_steps: usize = match public_input[OFFSET_LOG_N_STEPS].try_into() {
    //         Ok(n) => n,
    //         Err(_) => {
    //             return Err(VerifierError::InvalidProof(InvalidProof {
    //                 reason: "Number of steps is too large.".to_string(),
    //             }));
    //         }
    //     };
    //     require!(log_n_steps < 50, "Number of steps is too large.");
    //     ctx[MM_LOG_N_STEPS] = U256::from(log_n_steps);
    //     let log_trace_length = log_n_steps + LOG_CPU_COMPONENT_HEIGHT;

    //     // Range check limits.
    //     ctx[MM_RANGE_CHECK_MIN] = public_input[OFFSET_RC_MIN];
    //     ctx[MM_RANGE_CHECK_MAX] = public_input[OFFSET_RC_MAX];
    //     require!(
    //         ctx[MM_RANGE_CHECK_MIN] <= ctx[MM_RANGE_CHECK_MAX],
    //         "rc_min must be <= rc_max"
    //     );
    //     require!(
    //         ctx[MM_RANGE_CHECK_MAX] < ctx[MM_OFFSET_SIZE],
    //         "rc_max out of range"
    //     );

    //     // Layout.
    //     require!(
    //         public_input[OFFSET_LAYOUT_CODE] == LAYOUT_CODE,
    //         "Layout code mismatch."
    //     );

    //     // Initial and final pc ("program" memory segment).
    //     ctx[MM_INITIAL_PC] = public_input[OFFSET_PROGRAM_BEGIN_ADDR];
    //     ctx[MM_FINAL_PC] = public_input[OFFSET_PROGRAM_STOP_PTR];
    //     // Invalid final pc may indicate that the program end was moved, or the program didn't
    //     // complete.
    //     require!(ctx[MM_INITIAL_PC] == INITIAL_PC, "Invalid initial pc");
    //     require!(ctx[MM_FINAL_PC] == FINAL_PC, "Invalid final pc");

    //     // Initial and final ap ("execution" memory segment).
    //     ctx[MM_INITIAL_AP] = public_input[OFFSET_EXECUTION_BEGIN_ADDR];
    //     ctx[MM_FINAL_AP] = public_input[OFFSET_EXECUTION_STOP_PTR];

    //     // Public memory.
    //     let public_memory_pages_number: usize =
    //         match public_input[OFFSET_N_PUBLIC_MEMORY_PAGES].try_into() {
    //             Ok(n) => n,
    //             Err(_) => {
    //                 return Err(VerifierError::InvalidProof(InvalidProof {
    //                     reason: "Invalid number of memory pages.".to_string(),
    //                 }));
    //             }
    //         };
    //     require!(
    //         public_memory_pages_number >= 1 && public_memory_pages_number < 100000,
    //         "Invalid number of memory pages."
    //     );
    //     ctx[MM_N_PUBLIC_MEM_PAGES] = U256::from(public_memory_pages_number);

    //     {
    //         let mut n_public_memory_entries: U256 = U256::ZERO;
    //         for page_index in 0..public_memory_pages_number {
    //             let n_page_entries: U256 = public_input[offset_page_size(page_index)];
    //             require!(
    //                 n_page_entries < PUBLIC_MP_LIMIT,
    //                 "Invalid number of memory entries."
    //             );
    //             n_public_memory_entries += n_page_entries;
    //         }
    //         ctx[MM_N_PUBLIC_MEM_ENTRIES] = n_public_memory_entries;
    //     }

    //     require!(
    //         public_input_length(public_memory_pages_number) == public_input.len(),
    //         "Public input length mismatch."
    //     );

    //     // TODO: implement
    //     Ok((vec![], 0))
    // }

    // fn validate_fri_params(
    //     &self,
    //     fri_step_sizes: &[U256],
    //     log_fri_last_layer_deg_bound: U256,
    // ) -> Result<(), VerifierError> {
    //     Ok(())
    // }

    // fn init_verifier_params(
    //     &self,
    //     public_input: &[U256],
    //     proof_params: &[U256],
    // ) -> Result<(), VerifierError> {
    //     require!(
    //         proof_params.len() >= PROOF_PARAMS_FRI_STEPS_OFFSET,
    //         "Invalid proof params"
    //     );
    //     let n_fri_steps: usize = match proof_params[PROOF_PARAMS_N_FRI_STEPS_OFFSET].try_into() {
    //         Ok(n) => n,
    //         Err(_) => {
    //             return Err(VerifierError::InvalidProof(InvalidProof {
    //                 reason: "Invalid proof params".to_string(),
    //             }));
    //         }
    //     };
    //     require!(
    //         proof_params.len() == PROOF_PARAMS_FRI_STEPS_OFFSET + n_fri_steps,
    //         "Invalid proof params"
    //     );
    //     let log_blow_factor: U256 = proof_params[PROOF_PARAMS_LOG_BLOWUP_FACTOR_OFFSET];
    //     require!(
    //         log_blow_factor.as_limbs()[0] < 16,
    //         "logBlowupFactor must be at most 16"
    //     );
    //     require!(
    //         log_blow_factor.as_limbs()[0] > 1,
    //         "logBlowupFactor must be at least 1"
    //     );
    //     let proof_of_work_bits: U256 = proof_params[PROOF_PARAMS_PROOF_OF_WORK_BITS_OFFSET];
    //     require!(
    //         proof_of_work_bits.as_limbs()[0] < 50,
    //         "proofOfWorkBits must be at most 50"
    //     );
    //     require!(
    //         proof_of_work_bits.as_limbs()[0] > 1,
    //         "proofOfWorkBits must be at least 1"
    //     );

    //     let log_fri_last_layer_deg_bound: U256 =
    //         proof_params[PROOF_PARAMS_FRI_LAST_LAYER_LOG_DEG_BOUND_OFFSET];
    //     require!(
    //         log_fri_last_layer_deg_bound.as_limbs()[0] < 10,
    //         "logFriLastLayerDegBound must be at most 10"
    //     );
    //     let n_fri_steps: usize = proof_params[PROOF_PARAMS_N_FRI_STEPS_OFFSET]
    //         .try_into()
    //         .unwrap();
    //     require!(n_fri_steps <= MAX_FRI_STEPS, "Too many fri steps");
    //     require!(n_fri_steps > 1, "Not enough fri steps");
    //     let fri_step_sizes: Vec<U256> = (0..n_fri_steps)
    //         .map(|i| proof_params[PROOF_PARAMS_FRI_STEPS_OFFSET + i])
    //         .collect();
    //     let (ctx, log_trace_length) = self.air_specific_init(public_input)?;

    //     self.validate_fri_params(&fri_step_sizes, log_fri_last_layer_deg_bound)?;
    //     // let mut ctx = vec![U256::ZERO; CONTEXT_SIZE]; // to be defined
    //     Ok(())
    // }

    fn has_interaction(&self) -> bool {
        true // CPUVerifier returns always true(For the purpose of our work only)
    }

    // pub fn verify_proof(
    //     &self,
    //     proof_params: &[U256],
    //     proof: &[U256],
    //     public_input: &[U256],
    // ) -> Result<(), VerifierError> {
    //     let ctx = self.init_verifier_params(public_input, proof_params)?;
    //     let channel_ptr = get_channel_ptr();

    //     init_channel(
    //         &ctx,
    //         &channel_ptr,
    //         &get_public_input_hash(public_input)
    //     );

    //     ctx[MM_TRACE_COMMITMENT] = read_hash(&proof, &ctx, &channel_ptr, true);

    //     if self.has_interaction() {
    //         send_field_elements(&ctx, &channel_ptr, get_n_interaction_elements(), get_mm_interaction_elements());
    //         ctx[MM_TRACE_COMMITMENT + 1] = read_hash(&proof, &ctx, &channel_ptr, true);
    //     }

    //     send_field_elements(&ctx, &channel_ptr, 1, MM_COMPOSITION_ALPHA);

    //     ctx[MM_OODS_COMMITMENT] = read_hash(&proof, &ctx, &channel_ptr, true);

    //     send_field_elements(&ctx, &channel_ptr, 1, MM_OODS_POINT);

    //     let lmm_oods_values = get_mm_oods_values();
    //     for i in lmm_oods_values..lmm_oods_values + get_n_oods_values() {
    //         ctx[i] = read_field_element(&proof, &ctx, &channel_ptr, true);
    //     }

    //     oods_consistency_check(&ctx);

    //     send_field_elements(&ctx, &channel_ptr, 1, MM_OODS_ALPHA);

    //     ctx[MM_FRI_COMMITMENTS] = read_hash(&proof, &ctx, &channel_ptr, true);
        
    //     let n_fri_steps = get_fri_step_sizes(&ctx).len();
    //     let fri_eval_point_ptr = 295 // Lyubo: Use MM_FRI_EVAL_POINTS;
    //     for i in 1..n_fri_steps - 1 {
    //         send_field_elements(&ctx, &channel_ptr, 1, fri_eval_point_ptr + i);
    //         ctx[MM_FRI_COMMITMENTS + i] = read_hash(&proof, &ctx, &channel_ptr, true);
    //     }

    //     // Send last random FRI evaluation point.
    //     send_field_elements(&ctx, &channel_ptr, 1, 295 + n_fri_steps - 1);

    //     // Read FRI last layer commitment.
    //     read_last_fri_layer(&ctx);

    //     // Generate queries.
    //     verify_proof_of_work(&ctx, &channel_ptr, ctx[MM_PROOF_OF_WORK_BITS]);

    //     // Lyubo:FRI_QUEUE_SLOT_SIZE_IN_BYTES should be FRI_QUEUE_SLOT_SIZE and define it
    //     ctx[MM_N_UNIQUE_QUERIES] = send_random_queries(&ctx, &channel_ptr, ctx[MM_N_UNIQUE_QUERIES], ctx[MM_EVAL_DOMAIN_SIZE] - 1, MM_FRI_QUEUE, 3);

    //     self.compute_first_fri_layer(&ctx);

    //     fri_verify_layers(&ctx, &proof);
    //     Ok(())
    // }

    // Lyubo: Consider if stylus requires self in the function signature
    pub fn read_last_fri_layer(proof: &mut [U256], ctx: &mut [U256]) -> Result<(), String> {
        let lmm_channel = 10; // Lyubo: Use MM_CHANNEL;
        let fri_last_layer_deg_bound = ctx[315].to::<usize>(); // Lyubo: Use MM_FRI_LAST_LAYER_DEG_BOUND;
        let mut bad_input = U256::ZERO;

        let channel_ptr = lmm_channel;
        let last_layer_ptr = ctx[channel_ptr].to::<usize>();

        let last_layer_end = last_layer_ptr + fri_last_layer_deg_bound;
        for i in last_layer_ptr..last_layer_end {
            if proof[i] > PRIME_MINUS_ONE {
                bad_input |= U256::from(1);
            } else {
                bad_input |= U256::ZERO;
            }
        }

        let new_digest_ptr = last_layer_ptr - 1;
        let digest_ptr = channel_ptr + 1;
        proof[new_digest_ptr] = ctx[digest_ptr] + U256::from(1);

        let mut input_data = Vec::new();
        for i in new_digest_ptr..new_digest_ptr + fri_last_layer_deg_bound + 1 {
            input_data.extend_from_slice(&proof[i].to_be_bytes::<32>());
        }

        ctx[digest_ptr] = uint!(keccak(&input_data).into());
        ctx[channel_ptr + 2] = U256::ZERO;
        ctx[channel_ptr] = U256::from(last_layer_end);

        require!(bad_input == U256::ZERO, "Invalid field element.");
        ctx[316] = U256::from(last_layer_ptr); // Lyubo: Use MM_FRI_LAST_LAYER_PTR;
        Ok(())
    }

    // // Lyubo: Move to utils.rs
    // fn u256_to_bytes(value: U256) -> FixedBytes<32> {
    //     let mut buf = [0u8; 32];
    //     value.to_big_endian(&mut buf);
    //     FixedBytes::<32>::from(buf)
    // }

    // fn compute_first_fri_layer(&self, ctx: &[U256]) {
    //     adjust_query_indices_and_prepare_eval_points(ctx);
    //     // Lyubo: Use constants;
    //     read_query_responses_and_decommit(ctx, 12, 9, MM_TRACE_QUERY_RESPONSES, u256_to_bytes(ctx[MM_TRACE_COMMITMENT]));
    //     if self.has_interaction() {
    //         // Lyubo: Use constants;
    //         read_query_responses_and_decommit(ctx, 12, 3, MM_TRACE_QUERY_RESPONSES + 9, u256_to_bytes(ctx[MM_TRACE_COMMITMENT + 1]));
    //     }
    //     read_query_responses_and_decommit(ctx, 2, 2, MM_COMPOSITION_QUERY_RESPONSES, u256_to_bytes(ctx[MM_OODS_COMMITMENT]));

    //     // Lyubo: How to handler reverts? "?" sign?
    //     ctx[MM_FRI_QUEUE] = U256::from_be_slice(&static_call(
    //         Call::new_in(self), 
    //         oodsContractAddress,
    //         &ctx
    //     ));
    // }

    // fn adjust_query_indices_and_prepare_eval_points(ctx: &[U256]) {
    //     let n_unique_queries = ctx[MM_N_UNIQUE_QUERIES];
    //     let fri_queue = MM_FRI_QUEUE;
    //     let fri_queue_end = fri_queue + n_unique_queries;

    //     let eval_points_ptr = MM_OODS_EVAL_POINTS;
    //     let log_eval_domain_size = ctx[MM_LOG_EVAL_DOMAIN_SIZE];
    //     let eval_domain_size = ctx[MM_EVAL_DOMAIN_SIZE];
    //     let eval_domain_generator = ctx[MM_EVAL_DOMAIN_GENERATOR];

    //     for i in fri_queue..fri_queue_end {
    //         let query_idx = ctx[i];
    //         let adjusted_query_idx = query_idx + eval_domain_size;
    //         ctx[i] = adjusted_query_idx;
    //         // Lyubo: Use prime_field_element0::bit_reverse and prime_field_element0::expmod
    //         ctx[eval_points_ptr] = expmod(eval_domain_generator, bit_reverse(query_idx, log_eval_domain_size), K_MODULUS);
    //         eval_points_ptr += 1;
    //     }
    // }

    // // Lyubo: pass proof as it work with proof data, not only the ctx
    // fn read_query_responses_and_decommit(ctx: &[U256], n_total_columns: U256, n_columns: U256, proof_data_ptr: U256, merkle_root: FixedBytes<32>) {
    //     require!(n_columns <= n_total_columns, b"Too many columns.");

    //     let n_unique_queries = ctx[MM_N_UNIQUE_QUERIES];
    //     let channel_ptr = MM_CHANNEL;
    //     let fri_queue = MM_FRI_QUEUE;
    //     let fri_queue_end = fri_queue + n_unique_queries;
    //     let merkle_queue_ptr = MM_MERKLE_QUEUE;
    //     let row_size = n_columns;
    //     let proof_data_skip_bytes = n_total_columns - n_columns;

    //     let proof_ptr = ctx[channel_ptr];
    //     let merkle_ptr = merkle_queue_ptr;

    //     let mut input_data = Vec::with_capacity(row_size * 32);
    //     for i in fri_queue..fri_queue_end {
    //         for j in proof_ptr..proof_ptr + row_size {
    //             input_data.splice(j * 32..j * 32 + 32, ctx[j].to_be_bytes::<32>());
    //         }

    //         let merkle_leaf = uint!(keccak(&input_data).into()).bitand(COMMITMENT_MASK);
    //         if row_size == 1 {
    //             merkle_leaf = ctx[proof_ptr];
    //         } 

    //         ctx[merkle_ptr] = ctx[i];
    //         ctx[merkle_ptr + 1] = merkle_leaf;
    //         merkle_ptr += 2;

    //         for j in proof_ptr..proof_ptr + row_size {
    //             ctx[proof_data_ptr] = ctx[j];
    //             proof_data_ptr += 1;
    //         }
    //         proof_data_ptr += proof_data_skip_bytes;
    //     }
    //     ctx[channel_ptr] = proof_ptr + row_size;
    //     verify_merkle(channel_ptr, merkle_queue_ptr, merkle_root, n_unique_queries);
    // }

    // pub fn layout_specific_init(
    //     ctx: &mut [U256],
    //     public_input: &[U256],
    // ) -> Result<(), VerifierError> {
    //     // Output memory segment
    //     let output_begin_addr = public_input[OFFSET_OUTPUT_BEGIN_ADDR];
    //     let output_stop_ptr = public_input[OFFSET_OUTPUT_STOP_PTR];
    //     require!(
    //         output_begin_addr <= output_stop_ptr,
    //         "output begin_addr must be <= stop_ptr"
    //     );
    //     require!(
    //         output_stop_ptr < U256::from(1u64 << 64),
    //         "Out of range output stop_ptr."
    //     );

    //     // Number of steps: nSteps = 2 ** ctx[MM_LOG_N_STEPS]
    //     let n_steps: u64 = 1u64 << ctx[MM_LOG_N_STEPS].as_limbs()[0];

    //     // Pedersen segment
    //     ctx[MM_INITIAL_PEDERSEN_ADDR] = public_input[OFFSET_PEDERSEN_BEGIN_ADDR];
    //     validate_builtin_pointers(
    //         ctx[MM_INITIAL_PEDERSEN_ADDR],
    //         public_input[OFFSET_PEDERSEN_STOP_PTR],
    //         PEDERSEN_BUILTIN_RATIO,
    //         3,
    //         n_steps,
    //         "pedersen",
    //     )?;
    //     ctx[MM_PEDERSEN__SHIFT_POINT_X] = U256::from_dec_str(
    //         "33687124423693715171915430071063099500986961888935427573381850237373330333060",
    //     )
    //     .unwrap();
    //     ctx[MM_PEDERSEN__SHIFT_POINT_Y] = U256::from_dec_str(
    //         "27394515336187399075753500504026149502213372116606836955126321309182979900970",
    //     )
    //     .unwrap();

    //     // Range Check segment
    //     ctx[MM_INITIAL_RC_ADDR] = public_input[OFFSET_RANGE_CHECK_BEGIN_ADDR];
    //     validate_builtin_pointers(
    //         ctx[MM_INITIAL_RC_ADDR],
    //         public_input[OFFSET_RANGE_CHECK_STOP_PTR],
    //         RC_BUILTIN_RATIO,
    //         1,
    //         n_steps,
    //         "range_check",
    //     )?;
    //     ctx[MM_RC16__PERM__PUBLIC_MEMORY_PROD] = U256::ONE;

    //     // ECDSA segment
    //     ctx[MM_INITIAL_ECDSA_ADDR] = public_input[OFFSET_ECDSA_BEGIN_ADDR];
    //     validate_builtin_pointers(
    //         ctx[MM_INITIAL_ECDSA_ADDR],
    //         public_input[OFFSET_ECDSA_STOP_PTR],
    //         ECDSA_BUILTIN_RATIO,
    //         2,
    //         n_steps,
    //         "ecdsa",
    //     )?;
    //     ctx[MM_ECDSA__SIG_CONFIG_ALPHA] = U256::ONE;
    //     ctx[MM_ECDSA__SIG_CONFIG_BETA] = U256::from_dec_str(
    //         "50327761059496465184197731610822004429287718493293194845510360530958580779657",
    //     )
    //     .unwrap();
    //     ctx[MM_ECDSA__SIG_CONFIG_SHIFT_POINT_X] = U256::from_dec_str(
    //         "33687124423693715171915430071063099500986961888935427573381850237373330333060",
    //     )
    //     .unwrap();
    //     ctx[MM_ECDSA__SIG_CONFIG_SHIFT_POINT_Y] = U256::from_dec_str(
    //         "27394515336187399075753500504026149502213372116606836955126321309182979900970",
    //     )
    //     .unwrap();

    //     // Bitwise segment
    //     ctx[MM_INITIAL_BITWISE_ADDR] = public_input[OFFSET_BITWISE_BEGIN_ADDR];
    //     validate_builtin_pointers(
    //         ctx[MM_INITIAL_BITWISE_ADDR],
    //         public_input[OFFSET_BITWISE_STOP_ADDR],
    //         BITWISE__RATIO,
    //         5,
    //         n_steps,
    //         "bitwise",
    //     )?;
    //     ctx[MM_DILUTED_CHECK__PERMUTATION__PUBLIC_MEMORY_PROD] = U256::ONE;
    //     ctx[MM_DILUTED_CHECK__FIRST_ELM] = U256::ZERO;

    //     // EC_OP segment
    //     ctx[MM_INITIAL_EC_OP_ADDR] = public_input[OFFSET_EC_OP_BEGIN_ADDR];
    //     validate_builtin_pointers(
    //         ctx[MM_INITIAL_EC_OP_ADDR],
    //         public_input[OFFSET_EC_OP_STOP_ADDR],
    //         EC_OP_BUILTIN_RATIO,
    //         7,
    //         n_steps,
    //         "ec_op",
    //     )?;
    //     ctx[MM_EC_OP__CURVE_CONFIG_ALPHA] = U256::ONE;

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod test_constants;

    #[motsu::test]
    fn test_read_last_fri_layer() {
        let mut proof = test_constants::get_proof();
        let mut ctx = test_constants::get_ctx_read_last_fri_layer();
        StarkVerifier::read_last_fri_layer(&mut proof, &mut ctx).unwrap();

        assert_eq!(ctx[10], uint!(268_U256));
        assert_eq!(ctx[11], uint!(101063039785234930674416911940782140361807536835453250352760633033315826439229_U256));
        assert_eq!(ctx[316], uint!(204_U256));
    }

}
