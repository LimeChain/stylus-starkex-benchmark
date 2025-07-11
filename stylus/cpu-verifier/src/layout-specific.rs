extern crate alloc;
use alloc::vec::Vec;
use macros::require;
use stylus_sdk::{
    alloy_primitives::{U256, uint, Address},
    prelude::*,
};

use crate::interfaces::IConstraint;
use crate::prime_field_element0::PrimeFieldElement0;

pub trait LayoutSpecific: Sized + TopLevelStorage + HostAccess {

    fn get_pedersen_points_x(&self) -> IConstraint;
    fn get_pedersen_points_y(&self) -> IConstraint;
    fn get_poseidon_poseidon_full_round_key0(&self) -> IConstraint;
    fn get_poseidon_poseidon_full_round_key1(&self) -> IConstraint;
    fn get_poseidon_poseidon_full_round_key2(&self) -> IConstraint;
    fn get_poseidon_poseidon_partial_round_key0(&self) -> IConstraint;
    fn get_poseidon_poseidon_partial_round_key1(&self) -> IConstraint;

    fn get_layout_info(&self) -> (U256, U256) {
        let public_memory_offset = U256::from(21);
        let selected_builtins = U256::from(
            (1 << 0)
                | (1 << 1)
                | (1 << 2)
                | (1 << 3)
                | (1 << 4)
                | (1 << 5),
        );
        (public_memory_offset, selected_builtins)
    }

    // fn layout_specific_init(ctx: &mut [U256], public_input: &[U256]) -> Result<(), Vec<u8>> {
    //     let output_begin_addr = public_input[9];
    //     let output_stop_ptr = public_input[10];
    //     require!(output_begin_addr <= output_stop_ptr, "output begin_addr must be <= stop_ptr");
    //     require!(output_stop_ptr < uint!(18446744073709551616_U256), "Out of range output stop_ptr.");

    //     let n_steps = U256::from(1) << ctx[1274];
    //     ctx[346] = public_input[11];
    //     Self::validate_builtin_pointers(ctx[346], public_input[12], U256::from(128), U256::from(3), n_steps)?;

    //     ctx[344] = uint!(2089986280348253421170679821480865132823066470938446095505822317253594081284_U256);
    //     ctx[345] = uint!(1713931329540660377023406109199410414810705867260802078187082345529207694986_U256);
    //     ctx[347] = public_input[13];
    //     Self::validate_builtin_pointers(ctx[347], public_input[14], U256::from(8), U256::from(1), n_steps)?;

    //     ctx[335] = U256::from(1);
    //     ctx[348] = public_input[15];
    //     Self::validate_builtin_pointers(ctx[348], public_input[16], U256::from(8), U256::from(5), n_steps)?;

    //     ctx[339] = U256::from(1);
    //     ctx[340] = U256::from(0);

    //     ctx[349] = public_input[17];
    //     Self::validate_builtin_pointers(ctx[349], public_input[18], U256::from(8), U256::from(6), n_steps)?;

    //     Ok(())
    // }

    // fn validate_builtin_pointers(
    //     initial_address: U256,
    //     stop_address: U256,
    //     builtin_ratio: U256,
    //     cells_per_instance: U256,
    //     n_steps: U256
    // ) -> Result<(), Vec<u8>> {
    //     require!(
    //         initial_address < uint!(18446744073709551616_U256),
    //         "Out of range begin_addr."
    //     );
    //     let max_stop_ptr = initial_address + cells_per_instance * Self::safe_div(n_steps, builtin_ratio)?;
    //     require!(
    //         initial_address <= stop_address && stop_address <= max_stop_ptr,
    //         "Invalid stop_ptr."
    //     );
    //     Ok(())
    // }

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

    fn prepare_for_oods_check(&mut self, ctx: &mut [U256]) -> Result<(), Vec<u8>> {
        let pedersen_points_x = self.get_pedersen_points_x();
        let pedersen_points_y = self.get_pedersen_points_y();
        let poseidon_poseidon_full_round_key0 = self.get_poseidon_poseidon_full_round_key0();
        let poseidon_poseidon_full_round_key1 = self.get_poseidon_poseidon_full_round_key1();
        let poseidon_poseidon_full_round_key2 = self.get_poseidon_poseidon_full_round_key2();
        let poseidon_poseidon_partial_round_key0 = self.get_poseidon_poseidon_partial_round_key0();
        let poseidon_poseidon_partial_round_key1 = self.get_poseidon_poseidon_partial_round_key1();

        let oods_point = ctx[351];
        let n_steps = U256::from(1) << ctx[1274];
        let n_pedersen_hash_copies = Self::safe_div(n_steps, U256::from(128))?;
        let z_point_pow_pedersen = PrimeFieldElement0::fpow(oods_point, n_pedersen_hash_copies);
        
        ctx[317] = pedersen_points_x.compute(&mut *self, z_point_pow_pedersen)?;
        ctx[318] = pedersen_points_y.compute(&mut *self, z_point_pow_pedersen)?;

        ctx[338] = ctx[355];
        ctx[341] = ctx[356];
        ctx[342] = ctx[357];
        ctx[343] = Self::compute_diluted_cumulative_value(ctx);
        
        let n_poseidon_hash_copies = Self::safe_div(U256::from(1) << ctx[1274], U256::from(8))?;
        let z_point_pow_poseidon = PrimeFieldElement0::fpow(oods_point, n_poseidon_hash_copies);

        ctx[319] = poseidon_poseidon_full_round_key0.compute(&mut *self, z_point_pow_poseidon)?;
        ctx[320] = poseidon_poseidon_full_round_key1.compute(&mut *self, z_point_pow_poseidon)?;
        ctx[321] = poseidon_poseidon_full_round_key2.compute(&mut *self, z_point_pow_poseidon)?;
        ctx[322] = poseidon_poseidon_partial_round_key0.compute(&mut *self, z_point_pow_poseidon)?;
        ctx[323] = poseidon_poseidon_partial_round_key1.compute(&mut *self, z_point_pow_poseidon)?;
        
        Ok(())
    }

    fn compute_diluted_cumulative_value(ctx: &[U256]) -> U256 {
        let z = ctx[341];
        let alpha = ctx[342];
        let diff_multiplier = U256::from(16);
        let mut diff_x = U256::from(diff_multiplier) - U256::from(2);
        let mut p = U256::from(1) + z;
        let mut q = U256::from(1);
        let mut x = U256::from(1);

        for _ in 1..16 {
            x = x.add_mod(diff_x, PrimeFieldElement0::K_MODULUS);
            diff_x = diff_x.mul_mod(diff_multiplier, PrimeFieldElement0::K_MODULUS);
            let x_p = x.mul_mod(p, PrimeFieldElement0::K_MODULUS);
            let y = p + z.mul_mod(x_p, PrimeFieldElement0::K_MODULUS);
            q = (q.mul_mod(y, PrimeFieldElement0::K_MODULUS) + x.mul_mod(x_p, PrimeFieldElement0::K_MODULUS)).add_mod(q, PrimeFieldElement0::K_MODULUS);
            p = p.mul_mod(y, PrimeFieldElement0::K_MODULUS);
        }
        let res = p.add_mod(q.mul_mod(alpha, PrimeFieldElement0::K_MODULUS), PrimeFieldElement0::K_MODULUS);
        res
    }

}