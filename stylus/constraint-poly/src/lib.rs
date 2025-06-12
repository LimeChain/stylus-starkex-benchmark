//!
//! The address `0x05` refers to a built-in EVM ModExp precompile contract.
//! https://eips.ethereum.org/EIPS/eip-198
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::alloy_primitives::{address, uint, Address, U256};
use stylus_sdk::call::{static_call, Call};
use stylus_sdk::stylus_core::calls::errors::Error;
use stylus_sdk::{prelude::*, ArbResult};

const PRIME: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256);

#[storage]
#[entrypoint]
pub struct Hello;

#[public]
impl Hello {
    fn user_main(_input: Vec<u8>) -> ArbResult {
        // Will print 'Stylus says: Hello Stylus!' on your local dev node
        // Be sure to add "debug" feature flag to your Cargo.toml file as
        // shown below.

        Ok(Vec::new())
    }
}

impl Hello {
    pub fn make_expmod_input(base: U256, exponent: U256) -> Vec<u8> {
        let mut input = Vec::new();

        // Length fields (32 bytes each)
        input.extend_from_slice(&U256::from(32).to_be_bytes::<32>()); // base length
        input.extend_from_slice(&U256::from(32).to_be_bytes::<32>()); // exponent length
        input.extend_from_slice(&U256::from(32).to_be_bytes::<32>()); // modulus length

        // Value fields (32 bytes each)
        input.extend_from_slice(&base.to_be_bytes::<32>()); // base value
        input.extend_from_slice(&exponent.to_be_bytes::<32>()); // exponent value
        input.extend_from_slice(&PRIME.to_be_bytes::<32>()); // modulus value (PRIME)

        input
    }
    pub fn expmod(base: U256, exponent: U256) -> Result<U256, Error> {
        let expmod_address = address!("0000000000000000000000000000000000000005");

        let result_bytes = static_call(
            Call::new(),
            expmod_address,
            &Self::make_expmod_input(base, exponent),
        )
        .expect("modexp precompile failed");
        if result_bytes.len() != 32 {
            return Err(Error::Revert(
                "modexp precompile returned invalid length".into(),
            ));
        }
        Ok(U256::from_be_slice(&result_bytes))
    }
}

// fn expmod(base: U256, exponent: U256, modulus: U256) -> U256 {
//     let mut input = vec![U256::from(32), U256::from(32), U256::from(32)];
//     input.push(base);
//     input.push(exponent);
//     input.push(modulus);

//     // Flatten to bytes and call precompile 0x05
//     let input_bytes = encode_u256_vec(&input); // You'll write this helper
//     let mut output = [0u8; 32];
//     let ok = stylus_sdk::syscalls::staticcall(0x05, &input_bytes, &mut output);
//     if !ok {
//         panic!("modexp precompile failed");
//     }

//     U256::from_be_bytes(output)
// }
#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::hex;
    use stylus_sdk::alloy_primitives::{uint, U256};
    #[motsu::test]
    fn test_expmod_input() {
        let expected1 = hex!("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200615233634ff4ea9d9ff89cf4a6460f382b32d679d3ef86c95d917661c7df5bf0800000000000010ffffffffffffffffffffffffffffffffffffffffffffffff0800000000000011000000000000000000000000000000000000000000000001");
        let expected2 = hex!("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002003d8d2c79e51225ca679e36b4795d34603148f22aa2da68432609f1d4586dbc300000000000000000000000000000000000000000000000000000000020000000800000000000011000000000000000000000000000000000000000000000001");
        let expected3 = hex!("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002004c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad00000000000000000000000000000000000000000000000000000000000080000800000000000011000000000000000000000000000000000000000000000001");

        let base = uint!(0x0615233634ff4ea9d9ff89cf4a6460f382b32d679d3ef86c95d917661c7df5bf_U256);
        let exponent =
            uint!(0x0800000000000010ffffffffffffffffffffffffffffffffffffffffffffffff_U256);

        let input = Hello::make_expmod_input(base, exponent);

        // Verify total length: 6 * 32 bytes = 192 bytes
        assert_eq!(input.len(), 192);

        // Compare with expected hex string
        assert_eq!(input, expected1);
        println!("Input hex: 0x{}", hex::encode(&input));

        let base2 = uint!(0x03d8d2c79e51225ca679e36b4795d34603148f22aa2da68432609f1d4586dbc3_U256);
        let exponent2 = uint!(0x2000000_U256);

        let input2 = Hello::make_expmod_input(base2, exponent2);

        // Verify format consistency
        assert_eq!(input2.len(), 192);
        println!("Input2 hex: 0x{}", hex::encode(&input2));

        assert_eq!(input2, expected2);

        let base3 = uint!(0x04c03c56aa26ea6f3642546d9e7ffac66134612ff2fbf523821048bb194e00ad_U256);
        let exponent3 = uint!(0x8000_U256);

        let input3 = Hello::make_expmod_input(base3, exponent3);

        println!("Input3 hex: 0x{}", hex::encode(&input3));
        assert_eq!(input3.len(), 192);
        assert_eq!(input3, expected3);
    }
}
