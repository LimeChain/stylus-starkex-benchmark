extern crate alloc;
use alloc::vec::Vec;

use stylus_sdk::call::{static_call, Call};
use stylus_sdk::alloy_primitives::{uint, address, U256};

pub struct PrimeFieldElement0 {}

impl PrimeFieldElement0 {
    pub const BOUND: U256 = uint!(0xf80000000000020f00000000000000000000000000000000000000000000001f_U256);
    pub const K_MODULUS: U256 = uint!(0x800000000000011000000000000000000000000000000000000000000000001_U256);
    pub const K_MONTGOMERY_R: U256 = uint!(0x7fffffffffffdf0ffffffffffffffffffffffffffffffffffffffffffffffe1_U256);
    pub const K_MONTGOMERY_R_INV: U256 = uint!(0x40000000000001100000000000012100000000000000000000000000000000_U256);
    
    pub fn from_montgomery(val: U256) -> U256 {
        val.mul_mod(Self::K_MONTGOMERY_R_INV, Self::K_MODULUS)
    }

    pub fn fmul(a: U256, b: U256) -> U256 {
        a.mul_mod(b, Self::K_MODULUS)
    }

    pub fn fadd(a: U256, b: U256) -> U256 {
        a.add_mod(b, Self::K_MODULUS)
    }

    pub fn fsub(a: U256, b: U256) -> U256 {
        a.add_mod(Self::K_MODULUS - b, Self::K_MODULUS)
    }


    pub fn fpow(val: U256, exp: U256) -> U256 {
        PrimeFieldElement0::expmod(val, exp, Self::K_MODULUS)
    }

    pub fn inverse(val: U256) -> U256 {
        PrimeFieldElement0::expmod(val, Self::K_MODULUS - U256::from(2), Self::K_MODULUS)
    }

    pub fn expmod(base: U256, exponent: U256, modulus: U256) -> U256 {
        base.pow_mod(exponent, modulus)
    }

    pub fn bit_reverse(value: U256, number_of_bits: usize) -> U256 {
        let mut res = value;
        res = ((res & uint!(6148914691236517205_U256)) << U256::from(2)) | (res & uint!(12297829382473034410_U256));
        res = ((res & uint!(7378697629483820646_U256)) << U256::from(4)) | (res & uint!(29514790517935282584_U256));
        res = ((res & uint!(8680820740569200760_U256)) << U256::from(8)) | (res & uint!(138893131849107212160_U256));
        res = ((res & uint!(9187483429707480960_U256)) << U256::from(16)) | (res & uint!(2351995758005115125760_U256));
        res = ((res & uint!(9223231301513871360_U256)) << U256::from(32)) | (res & uint!(604453686576013073448960_U256));
        res = ((res & uint!(9223372034707292160_U256)) << U256::from(64)) | (res & uint!(39614081247908796759917199360_U256));

        res = res >> U256::from(127 - number_of_bits);
        res
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[motsu::test]
    fn test_bit_reverse() {
        let res = PrimeFieldElement0::bit_reverse(uint!(523277972_U256), 32);
        assert_eq!(res, uint!(694750456_U256));
    }

}