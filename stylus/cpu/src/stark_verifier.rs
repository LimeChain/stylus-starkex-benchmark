// use crate::offsets::memory_map::*;
use offsets::memory_map::MM_CHANNEL;

use stylus_sdk::alloy_primitives::{Uint, U256};
use stylus_sdk::alloy_sol_types::sol;
use stylus_sdk::prelude::SolidityError;

macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($msg.to_vec());
        }
    };
}

// TODO: switch to alloy::sol!
sol! {
    error InvalidProof(string reason);
    error ComputationFailed(string reason);
}

#[derive(SolidityError)]
pub enum VerifierError {
    InvalidProof(InvalidProof),
    ComputationFailed(ComputationFailed),
}

// hardcoded values from the 0x42AF9498647Be47A256C9cc8278eE94473Cb7771 contract constructor
const MIN_POW_BITS: usize = 0;
const NUM_SECURITY_BITS: usize = 80;

const PROOF_PARAMS_N_QUERIES_OFFSET: usize = 0;
const PROOF_PARAMS_LOG_BLOWUP_FACTOR_OFFSET: usize = 1;
const PROOF_PARAMS_PROOF_OF_WORK_BITS_OFFSET: usize = 2;
const PROOF_PARAMS_FRI_LAST_LAYER_LOG_DEG_BOUND_OFFSET: usize = 3;
const PROOF_PARAMS_N_FRI_STEPS_OFFSET: usize = 4;
const PROOF_PARAMS_FRI_STEPS_OFFSET: usize = 5;

pub struct StarkVerifier {}

impl StarkVerifier {
    // ["11","6","30","6","8","0","3","3","3","3","3","3","2"]
    fn init_verifier_params(
        &self,
        public_input: &[U256],
        proof_params: &[U256],
    ) -> Result<(), VerifierError> {
        if proof_params.len() < PROOF_PARAMS_FRI_STEPS_OFFSET {
            return Err(VerifierError::InvalidProof(InvalidProof {
                reason: "Invalid proof params".to_string(),
            }));
        }
        let n_fri_steps: usize = match proof_params[PROOF_PARAMS_N_FRI_STEPS_OFFSET].try_into() {
            Ok(n) => n,
            Err(_) => {
                return Err(VerifierError::InvalidProof(InvalidProof {
                    reason: "Invalid proof params".to_string(),
                }));
            }
        };
        if proof_params.len() != PROOF_PARAMS_FRI_STEPS_OFFSET + n_fri_steps {
            return Err(VerifierError::InvalidProof(InvalidProof {
                reason: "Invalid proof params".to_string(),
            }));
        }
        let log_blow_factor: U256 = proof_params[PROOF_PARAMS_LOG_BLOWUP_FACTOR_OFFSET];
        if log_blow_factor.as_limbs()[0] >= 16 {
            return Err(VerifierError::InvalidProof(InvalidProof {
                reason: "logBlowupFactor must be at most 16".to_string(),
            }));
        }
        if log_blow_factor.as_limbs()[0] <= 1 {
            return Err(VerifierError::InvalidProof(InvalidProof {
                reason: "logBlowupFactor must be at least 1".to_string(),
            }));
        }
        let proof_of_work_bits: U256 = proof_params[PROOF_PARAMS_PROOF_OF_WORK_BITS_OFFSET];
        if proof_of_work_bits.as_limbs()[0] >= 50 {
            return Err(VerifierError::InvalidProof(InvalidProof {
                reason: "proofOfWorkBits must be at most 50".to_string(),
            }));
        }
        if proof_of_work_bits.as_limbs()[0] <= 1 {
            return Err(VerifierError::InvalidProof(InvalidProof {
                reason: "proofOfWorkBits must be at least 1".to_string(),
            }));
        }

        // let mut ctx = vec![U256::ZERO; CONTEXT_SIZE]; // to be defined
        Ok(())
    }
    pub fn verify_proof(
        &self,
        proof_params: &[U256],
        proof: &[U256],
        public_input: &[U256],
    ) -> Result<(), VerifierError> {
        Ok(())
    }
}

trait MemoryAccessUtils {
    //   function getChannelPtr(uint256[] memory ctx) internal pure returns (uint256) {
    //     uint256 ctxPtr;
    //     assembly {
    //         ctxPtr := add(ctx, 0x20)
    //     }
    //     return ctxPtr + MM_CHANNEL * 0x20;
    // }
    // fn get_channel_ptr(ctx: &[U256]) -> U256 {

    // }
}
