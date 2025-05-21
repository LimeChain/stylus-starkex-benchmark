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
    alloy_primitives::{Address, FixedBytes, I256, U256},
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

        let verifier =
            ICairoVerifierContract::new(self.cairo_verifiers.get(verifier_id_usize).unwrap());
        let call = Call::new_in(verifier);

        let result = verifier.get_layout_info(call);

        let (public_memory_offset, selected_builtins) = match result {
            Ok((public_memory_offset, selected_builtins)) => {
                (public_memory_offset, selected_builtins)
            }
            Err(e) => return Err("Failed to get layout info".as_bytes().to_vec()),
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

        Ok(())
    }
}

const PAGE_INFO_SIZE: usize = 3;

#[cfg(test)]
mod test {
    use super::*;
}
