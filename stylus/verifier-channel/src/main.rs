#![cfg_attr(not(feature = "export-abi"), no_main)]

#[cfg(not(feature = "export-abi"))]
#[no_mangle]
pub extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    verifier_channel::print_abi("MIT-OR-APACHE-2.0", "pragma solidity ^0.8.23;");
}
