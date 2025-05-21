//!
//! GPS Statement Verifier
//!
//! The following contract implements the GPS statement verifier example from Foundry.
//!
//! ```solidity
//! contract GpsStatementVerifier {
//!     uint256 public number;
//!     function setNumber(uint256 newNumber) public {
//!         number = newNumber;
//!     }
//!     function increment() public {
//!         number++;
//!     }
//! }
//! ```
//!
//! The program is ABI-equivalent with Solidity, which means you can call it from both Solidity and Rust.
//! To do this, run `cargo stylus export-abi`.
//!
//! Note: this code is a template-only and has not been audited.
//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{uint, Address, FixedBytes, I256, U256},
    call::{self, Call, MethodError},
    prelude::*,
    storage::*,
};
// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
// sol_storage! {
//     #[entrypoint]
//     pub struct GpsStatementVerifier {
//         uint256 number;
//     }
// }
macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($msg.as_bytes().to_vec());
        }
    };
}

sol_interface! {
    interface IMemoryPageFactRegistry {
        function registerRegularMemoryPage(uint256[] memory memory_pairs, uint256 z, uint256 alpha, uint256 prime) external returns (bytes32, uint256, uint256);
    }
    interface ICairoVerifierContract {
        function verifyProofExternal(uint256[] calldata proofParams, uint256[] calldata proof, uint256[] calldata publicInput) external;
        function getLayoutInfo() external view returns (uint256 publicMemoryOffset, uint256 selectedBuiltins);
    }

    interface IBootloaderProgram {
        function getCompiledProgram() external pure returns (uint256[542] memory);
    }
}

#[storage]
#[entrypoint]
pub struct GpsStatementVerifier {
    // anti-re-init guard
    initialized: StorageBool,
    memory_page_fact_registry: StorageAddress,
    bootloader_program: StorageAddress,
    hashed_supported_cairo_verifiers: StorageU256,
    simple_bootloader_program_hash: StorageU256,
    cairo_verifiers: StorageVec<StorageAddress>,
}

// sol_storage! {
//     #[entrypoint]
//     pub struct GpsSV {
//         bool initialized;
//         #[borrow]
//         IMemoryPageFactRegistry memory_page_fact_registry;
//         // Address bootloader_program;
//     }

//     pub struct MemoryFactRegistry {
//         mapping(bytes32 => bool) verified_fact;
//         bool any_fact_registered;
//     }
// }
/// Declare that `GpsStatementVerifier` is a contract with the following external methods.
#[public]
impl GpsStatementVerifier {
    pub fn init(
        &mut self,
        bootloader_program: Address,
        memory_page_fact_registry: Address,
        cairo_verifiers: Vec<Address>,
        hashed_supported_cairo_verifiers: U256,
        simple_bootloader_program_hash: U256,
    ) -> Result<(), Vec<u8>> {
        // fail if it has already run
        require!(!self.initialized.get(), "already initialized");

        self.bootloader_program.set(bootloader_program);
        self.memory_page_fact_registry
            .set(memory_page_fact_registry);
        self.hashed_supported_cairo_verifiers
            .set(hashed_supported_cairo_verifiers);
        self.simple_bootloader_program_hash
            .set(simple_bootloader_program_hash);

        // copy verifier list
        for addr in cairo_verifiers {
            self.cairo_verifiers.push(addr);
        }

        self.initialized.set(true);
        Ok(())
    }

    pub fn verify_proof_and_register(
        &mut self,
        proof: Vec<U256>,
        task_metadata: Vec<U256>,
        cairo_aux_input: Vec<U256>,
        cairo_verifier_id: U256,
    ) -> Result<(), Vec<u8>> {
        // fail if it has not been initialized
        require!(self.initialized.get(), "not initialized");

        let verifier_id_usize: usize = match cairo_verifier_id.try_into() {
            Ok(val) => val,
            Err(_) => return Err("cairoVerifierId does not fit in usize".as_bytes().to_vec()),
        };

        require!(
            verifier_id_usize < self.cairo_verifiers.len(),
            "cairoVerifierId is out of range."
        );
        let cairo_public_input: &[U256] = &cairo_aux_input[..cairo_aux_input.len() - 2];

        let verifier_call_result =
            ICairoVerifierContract::new(self.cairo_verifiers.get(verifier_id_usize).unwrap())
                .get_layout_info(&mut *self);

        // verifier.get_layout_info(context)

        let (public_memory_offset, mut selected_builtins) = match verifier_call_result {
            Ok(val) => val,
            Err(_e) => return Err("Failed to get layout info".as_bytes().to_vec()),
        };

        let public_memory_offset_usize: usize = match public_memory_offset.try_into() {
            Ok(val) => val,
            Err(_) => {
                return Err("publicMemoryOffset does not fit in usize"
                    .as_bytes()
                    .to_vec())
            }
        };

        require!(
            cairo_public_input.len() > public_memory_offset_usize,
            "Invalid cairoAuxInput length."
        );
        let public_memory_pages: &[U256] = &cairo_public_input[public_memory_offset_usize..];
        let n_pages: usize = public_memory_pages[0].try_into().unwrap();
        require!(n_pages < 10000, "Invalid nPages.");
        // Validate publicMemoryPages.length.
        // Each page has a page info and a cumulative product.
        // There is no 'page address' in the page info for page 0, but this 'free' slot is
        // used to store the number of pages.
        require!(
            public_memory_pages.len() == n_pages * (PAGE_INFO_SIZE + 1),
            "Invalid publicMemoryPages length."
        );
        let (public_memory_length, memory_hash, product) =
            match Self::register_public_memory_main_page(
                &mut *self,
                task_metadata,
                cairo_aux_input,
                &mut selected_builtins,
            ) {
                Ok(val) => val,
                Err(e) => return Err(e),
            };

        Ok(())
    }
}

impl GpsStatementVerifier {
    fn register_public_memory_main_page(
        &mut self,
        task_metadata: Vec<U256>,
        aux_input: Vec<U256>,
        selected_builtins: &mut U256,
    ) -> Result<(U256, U256, U256), Vec<u8>> {
        let n_tasks: usize = task_metadata[0].try_into().unwrap();
        require!(n_tasks < 2usize.pow(30), "Invalid number of tasks.");

        let public_memory_length =
            (PROGRAM_SIZE + 2 + N_MAIN_ARGS + N_MAIN_RETURN_VALUES + 2 + 1 + 2 * n_tasks);

        let mut public_memory: Vec<U256> = vec![U256::ZERO; public_memory_length];
        let mut offset = 0;
        // Bootloader handling starts here
        {
            let bootloader_program_res = IBootloaderProgram::new(self.bootloader_program.get())
                .get_compiled_program(&mut *self);

            let bootloader_program = match bootloader_program_res {
                Ok(val) => val,
                Err(_e) => return Err("Failed to get compiled program".as_bytes().to_vec()),
            };

            for i in 0..PROGRAM_SIZE {
                public_memory[offset] = U256::from(i + INITIAL_PC);
                public_memory[offset + 1] = bootloader_program[i];
                offset += 2;
            }
        }

        {
            let initial_fp = aux_input[OFFSET_EXECUTION_BEGIN_ADDR];
            require!(
                initial_fp.gt(&U256::from(2)),
                "Invalid execution begin address."
            );
            public_memory[offset + 0] = initial_fp - U256::from(2);
            public_memory[offset + 1] = initial_fp;
            // Make sure [initial_fp - 1] = 0.
            public_memory[offset + 2] = initial_fp - U256::from(1);
            public_memory[offset + 3] = U256::ZERO;
            offset += 4;

            let return_values_address =
                aux_input[OFFSET_EXECUTION_STOP_PTR] - U256::from(N_BUILTINS);
            let mut builtin_segment_info_offset = OFFSET_OUTPUT_BEGIN_ADDR;

            for i in 0..N_BUILTINS {
                // Write argument address.
                public_memory[offset] = initial_fp + U256::from(i);
                let return_value_offset = offset + 2 * N_BUILTINS;

                // Write return value address.
                public_memory[return_value_offset] = return_values_address + U256::from(i);

                // Write values.
                if selected_builtins.bit(0) {
                    // Set the argument to the builtin start pointer.
                    public_memory[offset + 1] = aux_input[builtin_segment_info_offset];
                    // Set the return value to the builtin stop pointer.
                    public_memory[return_value_offset + 1] =
                        aux_input[builtin_segment_info_offset + 1];
                    builtin_segment_info_offset += 2;
                } else {
                    // Builtin is not present in layout, set the argument value and return value to 0.
                    public_memory[offset + 1] = U256::ZERO;
                    public_memory[return_value_offset + 1] = U256::ZERO;
                }
                offset += 2;
                *selected_builtins = *selected_builtins >> 1;
            }
            require!(
                *selected_builtins == U256::ZERO,
                "SELECTED_BUILTINS_VECTOR_IS_TOO_LONG"
            );
            // Skip the return values which were already written.
            offset += 2 * N_BUILTINS;
        }

        // Program output.
        {
            let mut output_address = aux_input[OFFSET_OUTPUT_BEGIN_ADDR];
            // Force that memory[outputAddress] and memory[outputAddress + 1] contain the
            // bootloader config (which is 2 words size).
            public_memory[offset + 0] = output_address;
            public_memory[offset + 1] = self.simple_bootloader_program_hash.get();
            public_memory[offset + 2] = output_address + U256::from(1);
            public_memory[offset + 3] = self.hashed_supported_cairo_verifiers.get();
            // Force that memory[outputAddress + 2] = nTasks.
            public_memory[offset + 4] = output_address + U256::from(2);
            public_memory[offset + 5] = U256::from(n_tasks);
            offset += 6;
            output_address += U256::from(3);

            let mut task_metadata_slice = &task_metadata[METADATA_TASKS_OFFSET..];
            for task in 0..n_tasks {
                let output_size = task_metadata_slice[METADATA_OFFSET_TASK_OUTPUT_SIZE];
                require!(
                    U256::from(2) <= output_size && output_size < U256::from(1u64 << 30),
                    "Invalid task output size."
                );
                let program_hash = task_metadata_slice[METADATA_OFFSET_TASK_PROGRAM_HASH];
                let n_tree_pairs: usize = task_metadata_slice[METADATA_OFFSET_TASK_N_TREE_PAIRS]
                    .try_into()
                    .unwrap();
                require!(
                    1 <= n_tree_pairs && n_tree_pairs < 2usize.pow(20),
                    "Invalid number of pairs in the Merkle tree structure."
                );
                // Force that memory[outputAddress] = outputSize.
                public_memory[offset + 0] = output_address;
                public_memory[offset + 1] = output_size;
                // Force that memory[outputAddress + 1] = programHash.
                public_memory[offset + 2] = output_address + U256::from(1);
                public_memory[offset + 3] = program_hash;
                offset += 4;
                output_address += output_size;
                let start_index = METADATA_TASK_HEADER_SIZE + 2 * n_tree_pairs;
                task_metadata_slice = &task_metadata_slice[start_index..];
                require!(
                    task_metadata_slice.len() == 0,
                    "Invalid length of taskMetadata."
                );

                require!(
                    aux_input[OFFSET_OUTPUT_STOP_PTR] == output_address,
                    "Inconsistent program output length."
                );
            }
        }

        require!(
            public_memory.len() == offset,
            "Not all Cairo public inputs were written."
        );

        let z = aux_input[aux_input.len() - 2];
        let alpha = aux_input[aux_input.len() - 1];

        let addr = self.memory_page_fact_registry.get();
        let result = IMemoryPageFactRegistry::new(addr).register_regular_memory_page(
            &mut *self,
            public_memory,
            z,
            alpha,
            K_MODULUS,
        );

        let (fact_hash, memory_hash, product) = match result {
            Ok(val) => val,
            Err(e) => return Err("Failed to register memory page".as_bytes().to_vec()),
        };

        Ok((U256::from(public_memory_length), memory_hash, product))
    }
}

const PAGE_INFO_SIZE: usize = 3;
const PROGRAM_SIZE: usize = 542;
const N_BUILTINS: usize = 6;
const N_MAIN_ARGS: usize = N_BUILTINS;
const N_MAIN_RETURN_VALUES: usize = N_BUILTINS;

const INITIAL_PC: usize = 1;
const OFFSET_EXECUTION_BEGIN_ADDR: usize = 6;
const OFFSET_EXECUTION_STOP_PTR: usize = 7;
const OFFSET_OUTPUT_BEGIN_ADDR: usize = 8;
const OFFSET_OUTPUT_STOP_PTR: usize = 9;
const METADATA_TASKS_OFFSET: usize = 1;
const METADATA_OFFSET_TASK_OUTPUT_SIZE: usize = 0;
const METADATA_OFFSET_TASK_PROGRAM_HASH: usize = 1;
const METADATA_OFFSET_TASK_N_TREE_PAIRS: usize = 2;
const METADATA_TASK_HEADER_SIZE: usize = 3;

pub const K_MODULUS: U256 = U256::from_limbs(
    *uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256).as_limbs(),
);

#[cfg(test)]
mod test {
    use super::*;
}
