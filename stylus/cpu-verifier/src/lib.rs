#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

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

#[cfg(test)]
#[path = "tests/mock_inputs.rs"]
pub mod mock_inputs;

mod interfaces;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use macros::require;

use crate::stark_verifier::StarkVerifier;
use crate::layout_specific::LayoutSpecific;
use crate::public_memory_offset::PublicMemoryOffset;
use crate::prime_field_element0::PrimeFieldElement0;
use crate::fri_statement_verifier::FriStatementVerifier;
use crate::merkle_statement_verifier::MerkleStatementVerifier;
use crate::interfaces::{IMerkleStatement, IFriStatement, ICpuOods, IConstraint, IConstraintPoly};

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
    }
}

impl MerkleStatementVerifier for CpuVerifier {
    fn get_merkle_statement(&self) -> IMerkleStatement {
        IMerkleStatement { address: self.merkle_statement.get() }
    }
}

impl FriStatementVerifier for CpuVerifier {
    fn get_fri_statement(&self) -> IFriStatement {
        IFriStatement { address: self.fri_statement.get() }
    }
}

impl LayoutSpecific for CpuVerifier {
    fn get_pedersen_points_x(&self) -> Address {
        self.pedersen_points_x.get()
    }

    fn get_pedersen_points_y(&self) -> Address {
        self.pedersen_points_y.get()
    }

    fn get_poseidon_poseidon_full_round_key0(&self) -> Address{
        self.poseidon_poseidon_full_round_key0.get()
    }

    fn get_poseidon_poseidon_full_round_key1(&self) -> Address{
        self.poseidon_poseidon_full_round_key1.get()
    }

    fn get_poseidon_poseidon_full_round_key2(&self) -> Address{
        self.poseidon_poseidon_full_round_key2.get()
    }

    fn get_poseidon_poseidon_partial_round_key0(&self) -> Address{
        self.poseidon_poseidon_partial_round_key0.get()
    }

    fn get_poseidon_poseidon_partial_round_key1(&self) -> Address{
        self.poseidon_poseidon_partial_round_key1.get()
    }
    
}   

impl StarkVerifier for CpuVerifier {
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

        Self::layout_specific_init(&mut ctx, public_input)?;

        Ok((ctx, log_trace_length))
    }

    fn oods_consistency_check(&mut self, ctx: &mut [U256], public_input: &[U256]) -> Result<(), Vec<u8>> {
        CpuVerifier::verify_memory_page_facts(ctx, public_input);
        ctx[331] = ctx[352];
        ctx[332] = ctx[353];
        ctx[334] = ctx[354];
        
        let public_memory_prod = CpuVerifier::compute_public_memory_quotient(ctx, public_input)?;
        ctx[333] = public_memory_prod;

        self.prepare_for_oods_check(ctx)?;

        let mut calldata = Vec::new();
        for i in 317..551 {
            calldata.extend_from_slice(&ctx[i].to_be_bytes::<32>());
        }
        
        let constraint_poly_contract = self.constraint_poly.get();
        let composition_from_trace_value = U256::from_be_slice(&self.vm().call(&self, constraint_poly_contract, &calldata)?);
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

    fn get_oods_contract(&self) -> ICpuOods {
        ICpuOods { address: self.oods.get() }
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
        oods: Address,
    ) {
        self.constraint_poly.set(constraint_poly);
        self.pedersen_points_x.set(pedersen_points_x);
        self.pedersen_points_y.set(pedersen_points_y);
        self.poseidon_poseidon_full_round_key0.set(poseidon_poseidon_full_round_key0);
        self.poseidon_poseidon_full_round_key1.set(poseidon_poseidon_full_round_key1);
        self.poseidon_poseidon_full_round_key2.set(poseidon_poseidon_full_round_key2);
        self.poseidon_poseidon_partial_round_key0.set(poseidon_poseidon_partial_round_key0);
        self.poseidon_poseidon_partial_round_key1.set(poseidon_poseidon_partial_round_key1);
        self.oods.set(oods);
    }

    pub fn verify_proof_external(
        &mut self,
        proof_params: Vec<U256>,
        mut proof: Vec<U256>,
        public_input: Vec<U256>,
    ) -> Result<Vec<U256>, Vec<u8>> {
        self.verify_proof(&proof_params, &mut proof, &public_input)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;
    use crate::test_constants;
    use crate::mock_inputs;
    use test_utils::try_execute;
    use stylus_sdk::{
        alloy_primitives::{U256, uint},
    };

    use oods::Oods;
    use constraint_poly::ConstraintPoly;
    use constraint_poly_preparer::ConstraintPolyPreparer;
    use constraint_poly_finalizer::ConstraintPolyFinalizer;

    use pedersen_hp_x_c::PedersenHashPointsXColumn;
    use pedersen_hp_y_c::PedersenHashPointsYColumn;
    use poseidon_frk_0_col::PoseidonPoseidonFullRoundKey0Column;
    use poseidon_frk_1_col::PoseidonPoseidonFullRoundKey1Column;
    use poseidon_frk_2_col::PoseidonPoseidonFullRoundKey2Column;
    use poseidon_prk_0_col::PoseidonPoseidonPartialRoundKey0Column;
    use poseidon_prk_1_col::PoseidonPoseidonPartialRoundKey1Column;

    #[motsu::test]
    fn test_verify_proof_external() {
        let vm = TestVM::default();
        let cpu_constraint_poly = Address::from([1u8; 20]);
        let pedersen_hash_points_x = Address::from([2u8; 20]);
        let pedersen_hash_points_y = Address::from([3u8; 20]);
        let poseidon_poseidon_full_round_key0 = Address::from([4u8; 20]);
        let poseidon_poseidon_full_round_key1 = Address::from([5u8; 20]);
        let poseidon_poseidon_full_round_key2 = Address::from([6u8; 20]);
        let poseidon_poseidon_partial_round_key0 = Address::from([7u8; 20]);
        let poseidon_poseidon_partial_round_key1 = Address::from([8u8; 20]);
        let oods_contract = Address::from([9u8; 20]);

        
        let mut cpu_verifier = CpuVerifier::from(&vm);
        cpu_verifier.init(
            cpu_constraint_poly,
            pedersen_hash_points_x,
            pedersen_hash_points_y,
            poseidon_poseidon_full_round_key0,
            poseidon_poseidon_full_round_key1,
            poseidon_poseidon_full_round_key2,
            poseidon_poseidon_partial_round_key0,
            poseidon_poseidon_partial_round_key1,
            oods_contract
        );

        let proof = test_constants::get_proof();
        let proof_params = test_constants::get_proof_params();
        let public_input = test_constants::get_public_input();

        
        let data1   = uint!(2502371038239847331946845555940821891939660827069539886818086403686260021246_U256);
        let data2   = uint!(513761785516736576210258345954495650460389361631034617172115002511570125974_U256);

        let ok_ret_1 = uint!(2476435194882991550378205418214791165604712474576866766823810310226558062065_U256);
        let ok_ret_2 = uint!(1444533035788560090889078696321009507857064390212204404518903797387225515076_U256);
        let ok_ret_3 = uint!(1747952454919021766681010400995206390562374609324430906386085649753967957996_U256);
        let ok_ret_4 = uint!(1664257228653772301912891197477956780973260593455413394763471271235501957228_U256);
        let ok_ret_5 = uint!(1938976483485279484363264204509611131731729867572976629648616677903267220493_U256);
        let ok_ret_6 = uint!(1499007735260395255086346814066654016187033964386904667040298584658325794077_U256);
        let ok_ret_7 = uint!(2486570557154671379335084513491649861794821253711847039152551529444239535533_U256);

        vm.mock_call(pedersen_hash_points_x, data1.to_be_bytes::<32>().to_vec(), Ok(ok_ret_1.to_be_bytes::<32>().to_vec()));
        vm.mock_call(pedersen_hash_points_y, data1.to_be_bytes::<32>().to_vec(), Ok(ok_ret_2.to_be_bytes::<32>().to_vec()));
        vm.mock_call(poseidon_poseidon_full_round_key0, data2.to_be_bytes::<32>().to_vec(), Ok(ok_ret_3.to_be_bytes::<32>().to_vec()));
        vm.mock_call(poseidon_poseidon_full_round_key1, data2.to_be_bytes::<32>().to_vec(), Ok(ok_ret_4.to_be_bytes::<32>().to_vec()));
        vm.mock_call(poseidon_poseidon_full_round_key2, data2.to_be_bytes::<32>().to_vec(), Ok(ok_ret_5.to_be_bytes::<32>().to_vec()));
        vm.mock_call(poseidon_poseidon_partial_round_key0, data2.to_be_bytes::<32>().to_vec(), Ok(ok_ret_6.to_be_bytes::<32>().to_vec()));
        vm.mock_call(poseidon_poseidon_partial_round_key1, data2.to_be_bytes::<32>().to_vec(), Ok(ok_ret_7.to_be_bytes::<32>().to_vec()));

        let ok_ret_8 = uint!(418385936848047481955394383802376566758559844720385213367193474142660347628_U256);
        let constraint_poly_input = mock_inputs::get_constraint_poly_input();
        vm.mock_call(cpu_constraint_poly, constraint_poly_input, Ok(ok_ret_8.to_be_bytes::<32>().to_vec()));

        let result = try_execute!(cpu_verifier.verify_proof_external(
            proof_params,
            proof,
            public_input
        ));


        // let expected_result = test_constants::get_ctx_verify_proof_external();
        // for i in 0..result.len() {
        //     assert_eq!(result[i], expected_result[i]);
        // }
    }


    // #[motsu::test]
    // fn test_oods_consistency_check() {
    //     // let mut proof = test_constants::get_proof();
    //     // let mut ctx = test_constants::get_ctx_oods_consistency_check();
    //     // let public_input = test_constants::get_public_input();
    //     // Self::oods_consistency_check(&mut ctx, &public_input);
    // }

    // #[motsu::test]
    // fn test_air_specific_init() {
    //     let public_input = test_constants::get_public_input();
    //     let (ctx, log_trace_length) = try_execute!(CpuVerifier::air_specific_init(&public_input));
    //     let ctx_expected = test_constants::get_ctx_air_specific_init();

    //     for i in 0..ctx.len() {
    //         assert_eq!(ctx[i], ctx_expected[i]);
    //     }
    //     assert_eq!(log_trace_length, U256::from(26));
    // }

    // #[motsu::test]
    // fn test_init_verifier_params() {
    //     let public_input = test_constants::get_public_input();
    //     let proof_params = test_constants::get_proof_params();
    //     let (ctx, _) = try_execute!(CpuVerifier::init_verifier_params(&public_input, &proof_params));
    //     let ctx_expected = test_constants::get_ctx_init_verifier_params();
    //     for i in 0..ctx.len() {
    //         assert_eq!(ctx[i], ctx_expected[i]);
    //     }
    // }

    // #[motsu::test]
    // fn test_read_last_fri_layer() {
    //     let mut proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_read_last_fri_layer();
    //     try_execute!(CpuVerifier::read_last_fri_layer(&mut proof, &mut ctx));

    //     assert_eq!(ctx[10], uint!(268_U256));
    //     assert_eq!(ctx[11], uint!(101063039785234930674416911940782140361807536835453250352760633033315826439229_U256));
    //     assert_eq!(ctx[316], uint!(204_U256));
    // }

    // // // Lyubo: Should fix this test
    // // // #[motsu::test]
    // // // fn test_compute_first_fri_layer() {
    // // //     let mut proof = test_constants::get_proof();
    // // //     let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
    // // //     CpuVerifier::compute_first_fri_layer(&mut proof, &mut ctx);
    // // // }

    // #[motsu::test]
    // fn test_adjust_query_indices_and_prepare_eval_points() {
    //     let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
    //     CpuVerifier::adjust_query_indices_and_prepare_eval_points(&mut ctx);
    //     assert_eq!(ctx[553], uint!(3515892385904170702434114719646176958489529091479346127319408828731691841909_U256));
    //     assert_eq!(ctx[109], uint!(4818245268_U256));
    //     assert_eq!(ctx[139], uint!(8285752452_U256));
    // }

    // // Lyubo: Finish this test with asserts
    // #[motsu::test]
    // fn test_read_query_responses_and_decommit() {
    //     let mut proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_compute_first_fri_layer();
    //     CpuVerifier::adjust_query_indices_and_prepare_eval_points(&mut ctx);

    //     ctx[10] = U256::from(8584); // proof pointer
    //     let merkle_root = CpuVerifier::u256_to_bytes(ctx[6]);
    //     try_execute!(CpuVerifier::read_query_responses_and_decommit(&mut proof, &mut ctx, 12, 9, 602, merkle_root));
    // }

    // #[motsu::test]
    // fn test_compute_diluted_cumulative_value() {
    //     let ctx = test_constants::get_ctx_compute_diluted_cumulative_value();
    //     let diluted_cumulative_value = CpuVerifier::compute_diluted_cumulative_value(&ctx);
    //     assert_eq!(diluted_cumulative_value, uint!(1552215061468209516830163195514878071221879601444981698864155012436627340325_U256));
    // }

    // #[motsu::test]
    // fn test_layout_specific_init() {
    //     let mut ctx = test_constants::get_ctx_layout_specific_init();
    //     let public_input = test_constants::get_public_input();
    //     try_execute!(CpuVerifier::layout_specific_init(&mut ctx, &public_input));
    //     assert_eq!(ctx[346], uint!(2392152_U256));
    //     assert_eq!(ctx[344], uint!(2089986280348253421170679821480865132823066470938446095505822317253594081284_U256));
    //     assert_eq!(ctx[345], uint!(1713931329540660377023406109199410414810705867260802078187082345529207694986_U256));
    //     assert_eq!(ctx[347], uint!(2490456_U256));
    //     assert_eq!(ctx[335], uint!(1_U256));
    //     assert_eq!(ctx[348], uint!(3014744_U256));
    //     assert_eq!(ctx[339], uint!(1_U256));
    //     assert_eq!(ctx[340], uint!(0_U256));
    //     assert_eq!(ctx[349], uint!(5636184_U256));
    // }

    // #[motsu::test]
    // fn test_fri_verify_layers() {
    //     let mut fri_step_sizes = Vec::new();
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(3));
    //     fri_step_sizes.push(U256::from(2));
    //     let proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_fri_verify_layers();
    //     try_execute!(CpuVerifier::fri_verify_layers(&mut ctx, &proof, &fri_step_sizes));
    // }

    // #[motsu::test]
    // fn test_compute_last_layer_hash() {
    //     let proof = test_constants::get_proof();
    //     let mut ctx = test_constants::get_ctx_compute_last_layer_hash();
    //     let res = try_execute!(CpuVerifier::compute_last_layer_hash(&proof, &mut ctx, 11, U256::from(20)));
    //     assert_eq!(res, uint!(16162843800108123221986333459199870243499406093086027266637045595326264638953_U256));
    // }

    // #[motsu::test]
    // fn test_horner_eval() {
    //     let proof = test_constants::get_proof();
    //     let res = try_execute!(CpuVerifier::horner_eval(&proof, 204, uint!(261724642622844706275344931861363185671055404258368687742740457067613420050_U256), 64));
    //     assert_eq!(res, uint!(2139028133873562710792122920124178712162573015562878092221167762764054446737_U256));
    // }

}
