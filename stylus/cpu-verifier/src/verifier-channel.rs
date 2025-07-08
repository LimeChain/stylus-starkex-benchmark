extern crate alloc;
use alloc::vec::Vec;

use macros::require;
use crate::prime_field_element0::PrimeFieldElement0;

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256, uint},
    crypto::keccak,
};

pub struct VerifierChannel {}

impl VerifierChannel {

    pub fn get_prng_ptr(channel_ptr: usize) -> usize {
        channel_ptr + 1 // return next index
    }

    pub fn get_random_bytes(ctx: &mut [U256], prng_ptr: usize) -> U256 {
        let mut input_data = Vec::new();
        input_data.extend_from_slice(&ctx[prng_ptr].to_be_bytes::<32>());
        input_data.extend_from_slice(&ctx[prng_ptr + 1].to_be_bytes::<32>());
        let random_bytes = uint!(keccak(&input_data).into());
        ctx[prng_ptr + 1] = ctx[prng_ptr + 1] + U256::from(1);
        random_bytes
    }

    pub fn init_channel(
        ctx: &mut [U256],
        channel_ptr: usize,
        public_input_hash: &FixedBytes<32>,
    ) {
        ctx[channel_ptr] = U256::ZERO;

        let prng_ptr = VerifierChannel::get_prng_ptr(channel_ptr);
        ctx[prng_ptr] = U256::from_be_slice(public_input_hash.as_slice());
        ctx[prng_ptr + 1] = U256::ZERO;
    }

    pub fn read_hash(proof: &[U256], ctx: &mut [U256], channel_ptr: usize, mix: bool) -> U256 {
        VerifierChannel::read_bytes(proof, ctx, channel_ptr, mix)
    }

    pub fn read_bytes(proof: &[U256], ctx: &mut [U256], channel_ptr: usize, mix: bool) -> U256 {
        let proof_ptr = ctx[channel_ptr];
        let val = proof[proof_ptr.to::<usize>()];
        ctx[channel_ptr] = proof_ptr + U256::from(1);

        if mix {
            // Mix the bytes that were read into the state of the channel.
            let digest_ptr = channel_ptr + 1;
            let mut input_data = Vec::new();
            input_data.extend_from_slice(&(ctx[digest_ptr] + U256::from(1)).to_be_bytes::<32>());
            input_data.extend_from_slice(&val.to_be_bytes::<32>());
            ctx[digest_ptr] = uint!(keccak(&input_data).into());
        }

        val
    }

    pub fn read_bytes_from_ptr(proof: &[U256], ctx: &mut [U256], channel_ptr: usize, mix: bool) -> U256 {
        let proof_ptr = ctx[channel_ptr];
        let val = VerifierChannel::read_ptr(proof, proof_ptr.to::<usize>(), 8);
        ctx[channel_ptr] = proof_ptr + U256::from(32);

        if mix {
            // Mix the bytes that were read into the state of the channel.
            let digest_ptr = channel_ptr + 1;
            let mut input_data = Vec::new();
            input_data.extend_from_slice(&(ctx[digest_ptr] + U256::from(1)).to_be_bytes::<32>());
            input_data.extend_from_slice(&val.to_be_bytes::<32>());
            ctx[digest_ptr] = uint!(keccak(&input_data).into());
        }

        val
    }

    pub fn read_ptr(proof: &[U256], ptr: usize, offset: usize) -> U256 {
        let element_index = ptr / 32;
        
        if ptr % 32 == 0 {
            proof[element_index]
        } else {
            let bit_shift = offset * 8;
            let element1 = proof[element_index] << bit_shift;
            let element2 = proof[element_index + 1] >> (256 - bit_shift);
            element1 | element2
        }
    }

    pub fn send_field_elements(ctx: &mut [U256], channel_ptr: usize, n_elements: usize, target_ptr: usize) -> Result<(), Vec<u8>> {
        require!(
            n_elements < 16777216,
            "Overflow protection failed."
        );

        let digest_ptr = channel_ptr + 1;
        let counter_ptr = digest_ptr + 1;

        for i in 0..n_elements {
            let mut field_element = PrimeFieldElement0::BOUND;
            while field_element >= PrimeFieldElement0::BOUND {
                let mut input_data = Vec::new();
                input_data.extend_from_slice(&ctx[digest_ptr].to_be_bytes::<32>());
                input_data.extend_from_slice(&ctx[counter_ptr].to_be_bytes::<32>());
                
                field_element = uint!(keccak(&input_data).into());
                ctx[counter_ptr] = ctx[counter_ptr] + U256::from(1);
            }

            ctx[target_ptr + i] = field_element.mul_mod(PrimeFieldElement0::K_MONTGOMERY_R_INV, PrimeFieldElement0::K_MODULUS);
        }

        Ok(())
    }

    pub fn read_field_element(proof: &[U256], ctx: &mut [U256], channel_ptr: usize, mix: bool) -> U256 {
        PrimeFieldElement0::from_montgomery(VerifierChannel::read_bytes(proof, ctx, channel_ptr, mix))
    }

    pub fn verify_proof_of_work(proof: &[U256], ctx: &mut [U256], channel_ptr: usize, proof_of_work_bits: U256) -> Result<U256, Vec<u8>> {
        if proof_of_work_bits == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let digest = ctx[channel_ptr + 1];
        let mut input_data = Vec::new();
        input_data.extend_from_slice(&uint!(0x0123456789abcded000000000000000000000000000000000000000000000000_U256).to_be_bytes::<32>()[0..8]);
        input_data.extend_from_slice(&digest.to_be_bytes::<32>());
        input_data.push(proof_of_work_bits.to::<u8>());
        let hash = keccak(&input_data);

        let proof_ptr = ctx[channel_ptr];
        let nonce_bytes = &proof[proof_ptr.to::<usize>()].to_be_bytes::<32>()[0..8];

        let mut proof_data = Vec::new();
        proof_data.extend_from_slice(&hash.as_slice());
        proof_data.extend_from_slice(&nonce_bytes);
        let proof_of_work_digest = U256::from_be_bytes(keccak(&proof_data).into());

        let mut final_input_data = Vec::new();
        final_input_data.extend_from_slice(&(digest + U256::from(1)).to_be_bytes::<32>());
        final_input_data.extend_from_slice(&nonce_bytes);
        let final_hash = keccak(&final_input_data).into();
        
        ctx[channel_ptr + 1] = final_hash;
        ctx[channel_ptr + 2] = U256::ZERO;
        ctx[channel_ptr] = proof_ptr * U256::from(32) + U256::from(8); // 8 is the offset of the nonce

        let proof_of_work_threshold = U256::from(1) << U256::from(256 - proof_of_work_bits.to::<usize>());
        require!(proof_of_work_digest < proof_of_work_threshold, "Proof of work check failed.");

        Ok(proof_of_work_digest)
    }

    pub fn send_random_queries(ctx: &mut [U256], channel_ptr: usize, count: usize, mask: U256, queries_out_ptr: U256, stride: U256) -> Result<U256, Vec<u8>> {
        require!(mask < U256::from(1) << U256::from(64), "mask must be < 2**64.");

        let mut val = U256::from(0);
        let mut shift = U256::from(0);
        let mut end_ptr = queries_out_ptr;
        let shift_step = U256::from(64);

        for _ in 0..count {
            if shift == U256::ZERO {
                val = VerifierChannel::get_random_bytes(ctx, VerifierChannel::get_prng_ptr(channel_ptr));
                shift = U256::from(256);
            }
            shift -= shift_step;
            let query_idx = (val >> shift) & mask;
            let mut ptr = end_ptr;

            let mut curr = U256::MAX;

            while ptr > queries_out_ptr {
                curr = ctx[(ptr - stride).to::<usize>()];
                if query_idx >= curr {
                    break;
                }
                
                ctx[ptr.to::<usize>()] = curr;
                ptr = ptr - stride;
            }

            if query_idx != curr {
                ctx[ptr.to::<usize>()] = query_idx;
                end_ptr += stride;
            } else {
                while ptr < end_ptr {
                    ctx[ptr.to::<usize>()] = ctx[(ptr + stride).to::<usize>()];
                    ptr += stride;
                }
            }
        }

        Ok((end_ptr - queries_out_ptr) / stride)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_constants;
    use test_utils::try_execute;
    use stylus_sdk::{
        alloy_primitives::hex,
    };

    #[motsu::test]
    fn test_init_channel() {
        let mut ctx = test_constants::get_initial_ctx();
        let channel_ptr = 1158; // 37056 in momory bytes / 32 bytes slot = 1158 index
        let proof_ptr = U256::ZERO; // set to initial element of the proof
        let public_input_hash = FixedBytes::<32>::new(hex!("0xd88ed8fb0839acf23f19b620f6a41ff00d7164ba987013d305cee03df15c23d6"));
        VerifierChannel::init_channel(&mut ctx, channel_ptr, &public_input_hash);

        assert_eq!(ctx[channel_ptr], proof_ptr);
        assert_eq!(ctx[VerifierChannel::get_prng_ptr(channel_ptr)], U256::from_be_slice(public_input_hash.as_slice()));
        assert_eq!(ctx[VerifierChannel::get_prng_ptr(channel_ptr) + 1], U256::ZERO);
    }

    #[motsu::test]
    fn test_read_hash() {
        let mut ctx = test_constants::get_initial_ctx();
        let proof = test_constants::get_proof();

        let channel_ptr = 10; // channel pointer stored at index 10
        let public_input_hash = FixedBytes::<32>::new(hex!("0xd88ed8fb0839acf23f19b620f6a41ff00d7164ba987013d305cee03df15c23d6"));
        VerifierChannel::init_channel(&mut ctx, channel_ptr, &public_input_hash);
        let hash = VerifierChannel::read_hash(&proof, &mut ctx, channel_ptr, true);

        assert_eq!(hash, uint!(0xfac0468b20f41ae0141a3cb50b1a2a67a1edf14b000000000000000000000000_U256));
        assert_eq!(ctx[channel_ptr + 1], uint!(FixedBytes::<32>::new(hex!("0xc7f98c4d0d908b93e8a4a09fae4349214b31a0695c51f731045ac6d3e6584591")).into()));
    }

    #[motsu::test]
    fn test_send_field_elements() {
        let mut ctx = test_constants::get_initial_ctx();
        let proof = test_constants::get_proof();

        let channel_ptr = 10; // channel pointer stored at index 10
        let public_input_hash = FixedBytes::<32>::new(hex!("0xd88ed8fb0839acf23f19b620f6a41ff00d7164ba987013d305cee03df15c23d6"));
        VerifierChannel::init_channel(&mut ctx, channel_ptr, &public_input_hash);
        VerifierChannel::read_hash(&proof, &mut ctx, channel_ptr, true);
        try_execute!(VerifierChannel::send_field_elements(&mut ctx, channel_ptr, 6, 352));

        assert_eq!(ctx[352 + 5], uint!(2761062090909355957053556856369845710198035091980059981525761706280755242673_U256));
        assert_eq!(ctx[12], uint!(6_U256));
    }

    #[motsu::test]
    fn test_read_field_element() {
        let mut ctx = test_constants::get_ctx_read_field_element();
        let proof = test_constants::get_proof();

        let channel_ptr = 10; // channel pointer stored at index 10
        let field_element = VerifierChannel::read_field_element(&proof, &mut ctx, channel_ptr, true);

        assert_eq!(field_element, uint!(2275741833758504896470175047018174931800329388283154351626181925085386637685_U256));
    }

    #[motsu::test]
    fn test_verify_proof_of_work() {
        let proof = test_constants::get_proof();
        let mut ctx = test_constants::get_ctx_verify_proof_of_work();
        let channel_ptr = 10;
        let proof_of_work_bits = U256::from(30);
        let digest = try_execute!(VerifierChannel::verify_proof_of_work(&proof, &mut ctx, channel_ptr, proof_of_work_bits));
        
        assert_eq!(digest, uint!(68701743034517859773582383053539537045812776708763242499428650007182_U256));
        assert_eq!(ctx[channel_ptr + 1], uint!(0xf8b467ddd11de948f4bac33029ba1446e95dafb837e4fd7cc7e0e3a20501f39d_U256));
    }   

    #[motsu::test]
    fn test_send_random_queries() {
        let proof = test_constants::get_proof();
        let mut ctx = test_constants::get_ctx_verify_proof_of_work();
        let channel_ptr = 10;
        let mask = ctx[0] - U256::from(1);
        let proof_of_work_bits = U256::from(30);
        try_execute!(VerifierChannel::verify_proof_of_work(&proof, &mut ctx, channel_ptr, proof_of_work_bits));
        let result = try_execute!(VerifierChannel::send_random_queries(&mut ctx, channel_ptr, 11, mask, U256::from(109), U256::from(3)));
        assert_eq!(result, U256::from(11));
    }
}

