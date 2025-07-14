#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[cfg(test)]
#[path = "tests/test_constants.rs"]
pub mod test_constants;


#[path = "stark-verifier.rs"]
pub mod stark_verifier;
#[path = "layout-specific.rs"]
pub mod layout_specific;
#[path = "verifier-channel.rs"]
pub mod verifier_channel;
mod interfaces;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use utils::{
    require,
    prime_field_element0::PrimeFieldElement0,
    public_memory_offset::PublicMemoryOffset
};

use crate::stark_verifier::StarkVerifier;
use crate::layout_specific::LayoutSpecific;
use crate::interfaces::{IConstraint, IConstraintPoly, IInitVerifier, IFriStatementVerifier};

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256, uint, Address},
    crypto::keccak,
    prelude::*,
};


sol_storage! {
    #[entrypoint]
    pub struct CpuVerifier {
        address oods;
        address merkle_statement;
        address fri_statement;
        address constraint_poly;
        address pedersen_points_x;
        address pedersen_points_y;
        address poseidon_poseidon_full_round_key0;
        address poseidon_poseidon_full_round_key1;
        address poseidon_poseidon_full_round_key2;
        address poseidon_poseidon_partial_round_key0;
        address poseidon_poseidon_partial_round_key1;
        address init_verifier;
        address fri_statement_verifier;
    }
}

impl LayoutSpecific for CpuVerifier {
    fn get_pedersen_points_x(&self) -> IConstraint {
        IConstraint { address: self.pedersen_points_x.get() }
    }

    fn get_pedersen_points_y(&self) -> IConstraint {
        IConstraint { address: self.pedersen_points_y.get() }
    }

    fn get_poseidon_poseidon_full_round_key0(&self) -> IConstraint {
        IConstraint { address: self.poseidon_poseidon_full_round_key0.get() }
    }

    fn get_poseidon_poseidon_full_round_key1(&self) -> IConstraint {
        IConstraint { address: self.poseidon_poseidon_full_round_key1.get() }
    }

    fn get_poseidon_poseidon_full_round_key2(&self) -> IConstraint {
        IConstraint { address: self.poseidon_poseidon_full_round_key2.get() }
    }

    fn get_poseidon_poseidon_partial_round_key0(&self) -> IConstraint {
        IConstraint { address: self.poseidon_poseidon_partial_round_key0.get() }
    }

    fn get_poseidon_poseidon_partial_round_key1(&self) -> IConstraint {
        IConstraint { address: self.poseidon_poseidon_partial_round_key1.get() }
    }
    
}   

impl StarkVerifier for CpuVerifier {
    
    fn oods_consistency_check(&mut self, ctx: &mut [U256], public_input: &[U256]) -> Result<(), Vec<u8>> {
        CpuVerifier::verify_memory_page_facts(ctx, public_input);
        ctx[331] = ctx[352];
        ctx[332] = ctx[353];
        ctx[334] = ctx[354];
        
        let public_memory_prod = CpuVerifier::compute_public_memory_quotient(ctx, public_input)?;
        ctx[333] = public_memory_prod;

        self.prepare_for_oods_check(ctx)?;

        let constraint_poly_contract: IConstraintPoly = IConstraintPoly { address: self.constraint_poly.get() };
        let composition_from_trace_value = constraint_poly_contract.compute(&mut *self, ctx[317..551].to_vec())?;
        let claimed_composition = PrimeFieldElement0::fadd(ctx[551], PrimeFieldElement0::fmul(ctx[351], ctx[552]));
        require!(composition_from_trace_value == claimed_composition, "claimedComposition does not match trace");
        
        Ok(())
    }

    fn get_public_input_hash(public_input: &[U256]) -> FixedBytes<32> {
        let n_pages = public_input[21].to::<usize>();
        let offset_page_prod = PublicMemoryOffset::get_offset_page_prod(0, n_pages);
        let mut input_data = Vec::new();
        for i in 0..offset_page_prod {
            input_data.extend_from_slice(&public_input[i].to_be_bytes::<32>());
        }

        keccak(&input_data).into()
    }

    fn get_init_verifier(&self) -> IInitVerifier {
        IInitVerifier { address: self.init_verifier.get() }
    }

    fn get_fri_statement_verifier(&self) -> IFriStatementVerifier {
        IFriStatementVerifier { address: self.fri_statement_verifier.get() }
    }
}

impl CpuVerifier {

    pub fn verify_memory_page_facts(ctx: &[U256], public_input: &[U256]) {
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

    pub fn compute_public_memory_quotient(ctx: &[U256], public_input: &[U256]) -> Result<U256, Vec<u8>> {
        let n_values = ctx[1275];
        let z = ctx[331];
        let alpha = ctx[332];
        
        let public_memory_size = Self::safe_div(ctx[324], U256::from(16))?;
        require!(n_values < uint!(16777216_U256), "Overflow protection failed.");
        require!(n_values <= public_memory_size, "Number of values of public memory is too large.");

        let n_public_memory_pages = ctx[1276].to::<usize>();
        let cumulative_prods_ptr = ctx[5].to::<usize>() + PublicMemoryOffset::get_offset_page_prod(0, n_public_memory_pages);
        let denominator = Self::compute_public_memory_prod(public_input, cumulative_prods_ptr, n_public_memory_pages, PrimeFieldElement0::K_MODULUS);
        
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

    pub fn compute_public_memory_prod(public_input: &[U256], cumulative_prods_ptr: usize, n_public_memory_pages: usize, prime: U256) -> U256 {
        let mut res = U256::from(1);
        let last_ptr = cumulative_prods_ptr + n_public_memory_pages;
        for ptr in cumulative_prods_ptr..last_ptr {
            res = res.mul_mod(public_input[ptr], prime);
        }
        res
    }
}

#[public]
impl CpuVerifier {

    #[inline]
    pub fn init(
        &mut self,
        constraint_poly: Address,
        pedersen_points_x: Address,
        pedersen_points_y: Address,
        poseidon_poseidon_full_round_key0: Address,
        poseidon_poseidon_full_round_key1: Address,
        poseidon_poseidon_full_round_key2: Address,
        poseidon_poseidon_partial_round_key0: Address,
        poseidon_poseidon_partial_round_key1: Address,
        init_verifier: Address,
        fri_statement_verifier: Address,
    ) {
        self.constraint_poly.set(constraint_poly);
        self.pedersen_points_x.set(pedersen_points_x);
        self.pedersen_points_y.set(pedersen_points_y);
        self.poseidon_poseidon_full_round_key0.set(poseidon_poseidon_full_round_key0);
        self.poseidon_poseidon_full_round_key1.set(poseidon_poseidon_full_round_key1);
        self.poseidon_poseidon_full_round_key2.set(poseidon_poseidon_full_round_key2);
        self.poseidon_poseidon_partial_round_key0.set(poseidon_poseidon_partial_round_key0);
        self.poseidon_poseidon_partial_round_key1.set(poseidon_poseidon_partial_round_key1);
        self.init_verifier.set(init_verifier);
        self.fri_statement_verifier.set(fri_statement_verifier);
    }

    #[inline]
    pub fn verify_proof_external(
        &mut self,
        proof_params: Vec<U256>,
        mut proof: Vec<U256>,
        public_input: Vec<U256>,
    ) -> Result<Vec<U256>, Vec<u8>> {
        Ok(self.verify_proof(&proof_params, &mut proof, &public_input)?)
    }
}
