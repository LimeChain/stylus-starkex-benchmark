/*
  Copyright 2019-2024 StarkWare Industries Ltd.

  Licensed under the Apache License, Version 2.0 (the "License").
  You may not use this file except in compliance with the License.
  You may obtain a copy of the License at

  https://www.starkware.co/open-source-license/

  Unless required by applicable law or agreed to in writing,
  software distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions
  and limitations under the License.
*/
// ---------- The following code was auto-generated. PLEASE DO NOT EDIT. ----------
// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.6.12;

import "../../interfaces/IPeriodicColumn.sol";
import "./MemoryMap.sol";
import "./StarkParameters.sol";
import "./CpuPublicInputOffsets.sol";
import "../CairoVerifierContract.sol";

import {console} from "forge-std/console.sol";


abstract contract LayoutSpecific is MemoryMap, StarkParameters, CpuPublicInputOffsets, CairoVerifierContract {
    IPeriodicColumn pedersenPointsX;
    IPeriodicColumn pedersenPointsY;
    IPeriodicColumn poseidonPoseidonFullRoundKey0;
    IPeriodicColumn poseidonPoseidonFullRoundKey1;
    IPeriodicColumn poseidonPoseidonFullRoundKey2;
    IPeriodicColumn poseidonPoseidonPartialRoundKey0;
    IPeriodicColumn poseidonPoseidonPartialRoundKey1;

    function initPeriodicColumns(address[] memory auxPolynomials) internal {
        pedersenPointsX = IPeriodicColumn(auxPolynomials[1]);
        pedersenPointsY = IPeriodicColumn(auxPolynomials[2]);
        poseidonPoseidonFullRoundKey0 = IPeriodicColumn(auxPolynomials[3]);
        poseidonPoseidonFullRoundKey1 = IPeriodicColumn(auxPolynomials[4]);
        poseidonPoseidonFullRoundKey2 = IPeriodicColumn(auxPolynomials[5]);
        poseidonPoseidonPartialRoundKey0 = IPeriodicColumn(auxPolynomials[6]);
        poseidonPoseidonPartialRoundKey1 = IPeriodicColumn(auxPolynomials[7]);
    }

    function getLayoutInfo()
        external pure override returns (uint256 publicMemoryOffset, uint256 selectedBuiltins) {
        publicMemoryOffset = OFFSET_N_PUBLIC_MEMORY_PAGES;
        selectedBuiltins =
            (1 << OUTPUT_BUILTIN_BIT) |
            (1 << PEDERSEN_BUILTIN_BIT) |
            (1 << RANGE_CHECK_BUILTIN_BIT) |
            (1 << BITWISE_BUILTIN_BIT) |
            (1 << 7);
    }

    function safeDiv(uint256 numerator, uint256 denominator) internal pure returns (uint256) {
        require(denominator > 0, "The denominator must not be zero");
        require(numerator % denominator == 0, "The numerator is not divisible by the denominator.");
        return numerator / denominator;
    }

    function validateBuiltinPointers(
        uint256 initialAddress, uint256 stopAddress, uint256 builtinRatio, uint256 cellsPerInstance,
        uint256 nSteps, string memory builtinName)
        internal pure {
        require(
            initialAddress < 2**64,
            string(abi.encodePacked("Out of range ", builtinName, " begin_addr.")));
        uint256 maxStopPtr = initialAddress + cellsPerInstance * safeDiv(nSteps, builtinRatio);
        require(
            initialAddress <= stopAddress && stopAddress <= maxStopPtr,
            string(abi.encodePacked("Invalid ", builtinName, " stop_ptr.")));
    }

    function layoutSpecificInit(uint256[] memory ctx, uint256[] memory publicInput)
        internal pure {
        // "output" memory segment.
        uint256 outputBeginAddr = publicInput[OFFSET_OUTPUT_BEGIN_ADDR];
        uint256 outputStopPtr = publicInput[OFFSET_OUTPUT_STOP_PTR];
        require(outputBeginAddr <= outputStopPtr, "output begin_addr must be <= stop_ptr");
        require(outputStopPtr < 2**64, "Out of range output stop_ptr.");

        uint256 nSteps = 2 ** ctx[MM_LOG_N_STEPS];

        // "pedersen" memory segment.
        ctx[MM_INITIAL_PEDERSEN_ADDR] = publicInput[OFFSET_PEDERSEN_BEGIN_ADDR];
        validateBuiltinPointers(
            ctx[MM_INITIAL_PEDERSEN_ADDR], publicInput[OFFSET_PEDERSEN_STOP_PTR],
            PEDERSEN_BUILTIN_RATIO, 3, nSteps, 'pedersen');

        // Pedersen's shiftPoint values.
        ctx[MM_PEDERSEN__SHIFT_POINT_X] =
            0x49ee3eba8c1600700ee1b87eb599f16716b0b1022947733551fde4050ca6804;
        ctx[MM_PEDERSEN__SHIFT_POINT_Y] =
            0x3ca0cfe4b3bc6ddf346d49d06ea0ed34e621062c0e056c1d0405d266e10268a;

        // "range_check" memory segment.
        ctx[MM_INITIAL_RANGE_CHECK_ADDR] = publicInput[OFFSET_RANGE_CHECK_BEGIN_ADDR];
        validateBuiltinPointers(
            ctx[MM_INITIAL_RANGE_CHECK_ADDR], publicInput[OFFSET_RANGE_CHECK_STOP_PTR],
            RANGE_CHECK_BUILTIN_RATIO, 1, nSteps, 'range_check');
        ctx[MM_RANGE_CHECK16__PERM__PUBLIC_MEMORY_PROD] = 1;

        // "bitwise" memory segment.
        ctx[MM_INITIAL_BITWISE_ADDR] = publicInput[OFFSET_BITWISE_BEGIN_ADDR];
        validateBuiltinPointers(
            ctx[MM_INITIAL_BITWISE_ADDR], publicInput[OFFSET_BITWISE_STOP_PTR],
            BITWISE__RATIO, 5, nSteps, 'bitwise');

        ctx[MM_DILUTED_CHECK__PERMUTATION__PUBLIC_MEMORY_PROD] = 1;
        ctx[MM_DILUTED_CHECK__FIRST_ELM] = 0;

        // "poseidon" memory segment.
        ctx[MM_INITIAL_POSEIDON_ADDR] = publicInput[OFFSET_POSEIDON_BEGIN_ADDR];
        validateBuiltinPointers(
            ctx[MM_INITIAL_POSEIDON_ADDR], publicInput[OFFSET_POSEIDON_STOP_PTR],
            POSEIDON__RATIO, 6, nSteps, 'poseidon');
    }

    function prepareForOodsCheck(uint256[] memory ctx) internal view {
        uint256 oodsPoint = ctx[MM_OODS_POINT];
        uint256 nSteps = 2 ** ctx[MM_LOG_N_STEPS];

        // The number of copies in the pedersen hash periodic columns is
        // nSteps / PEDERSEN_BUILTIN_RATIO / PEDERSEN_BUILTIN_REPETITIONS.
        uint256 nPedersenHashCopies = safeDiv(
            nSteps,
            PEDERSEN_BUILTIN_RATIO * PEDERSEN_BUILTIN_REPETITIONS);
        uint256 zPointPowPedersen = fpow(oodsPoint, nPedersenHashCopies);
        
        ctx[MM_PERIODIC_COLUMN__PEDERSEN__POINTS__X] = pedersenPointsX.compute(zPointPowPedersen);
        ctx[MM_PERIODIC_COLUMN__PEDERSEN__POINTS__Y] = pedersenPointsY.compute(zPointPowPedersen);

        ctx[MM_DILUTED_CHECK__PERMUTATION__INTERACTION_ELM] = ctx[MM_INTERACTION_ELEMENTS +
            3];
        ctx[MM_DILUTED_CHECK__INTERACTION_Z] = ctx[MM_INTERACTION_ELEMENTS + 4];
        ctx[MM_DILUTED_CHECK__INTERACTION_ALPHA] = ctx[MM_INTERACTION_ELEMENTS +
            5];

        ctx[MM_DILUTED_CHECK__FINAL_CUM_VAL] = computeDilutedCumulativeValue(ctx);
        

        // The number of copies in the Poseidon hash periodic columns is
        // nSteps / POSEIDON__RATIO.
        uint256 nPoseidonHashCopies = safeDiv(
            2 ** ctx[MM_LOG_N_STEPS],
            POSEIDON__RATIO);
        uint256 zPointPowPoseidon = fpow(oodsPoint, nPoseidonHashCopies);
        
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY0] = poseidonPoseidonFullRoundKey0.compute(zPointPowPoseidon);
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY1] = poseidonPoseidonFullRoundKey1.compute(zPointPowPoseidon);
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__FULL_ROUND_KEY2] = poseidonPoseidonFullRoundKey2.compute(zPointPowPoseidon);
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__PARTIAL_ROUND_KEY0] = poseidonPoseidonPartialRoundKey0.compute(zPointPowPoseidon);
        ctx[MM_PERIODIC_COLUMN__POSEIDON__POSEIDON__PARTIAL_ROUND_KEY1] = poseidonPoseidonPartialRoundKey1.compute(zPointPowPoseidon);

    }

    /*
      Computes the final cumulative value of the diluted pool.
    */
    function computeDilutedCumulativeValue(uint256[] memory ctx)
        internal
        pure
        returns (uint256 res)
    {
        // The cumulative value is defined using the following recursive formula:
        //   r_1 = 1, r_{j+1} = r_j * (1 + z * u_j) + alpha * u_j^2 (for j >= 1)
        // where u_j = Dilute(j, spacing, n_bits) - Dilute(j-1, spacing, n_bits)
        // and we want to compute the final value r_{2^n_bits}.
        // Note that u_j depends only on the number of trailing zeros in the binary representation
        // of j. Specifically,
        //   u_{(1 + 2k) * 2^i} = u_{2^i} =
        //   u_{2^{i - 1}} + 2^{i * spacing} - 2^{(i - 1) * spacing + 1}.
        //
        // The recursive formula can be reduced to a nonrecursive form:
        //   r_j = prod_{n=1..j-1}(1 + z*u_n) +
        //     alpha * sum_{n=1..j-1}(u_n^2 * prod_{m=n + 1..j - 1}(1 + z * u_m))
        //
        // We rewrite this equation to generate a recursive formula that converges in log(j) steps:
        // Denote:
        //   p_i = prod_{n=1..2^i - 1}(1 + z * u_n)
        //   q_i = sum_{n=1..2^i - 1}(u_n^2 * prod_{m=n + 1..2^i-1}(1 + z * u_m))
        //   x_i = u_{2^i}.
        //
        // Clearly
        //   r_{2^i} = p_i + alpha * q_i.
        // Moreover, due to the symmetry of the sequence u_j,
        //   p_i = p_{i - 1} * (1 + z * x_{i - 1}) * p_{i - 1}
        //   q_i = q_{i - 1} * (1 + z * x_{i - 1}) * p_{i - 1} + x_{i - 1}^2 * p_{i - 1} + q_{i - 1}
        //
        // Now we can compute p_{n_bits} and q_{n_bits} in 'n_bits' steps and we are done.
        uint256 z = ctx[MM_DILUTED_CHECK__INTERACTION_Z];
        uint256 alpha = ctx[MM_DILUTED_CHECK__INTERACTION_ALPHA];
        uint256 diffMultiplier = 1 << DILUTED_SPACING;
        uint256 diffX = diffMultiplier - 2;
        // Initialize p, q and x to p_1, q_1 and x_0 respectively.
        uint256 p = 1 + z;
        uint256 q = 1;
        uint256 x = 1;
        assembly {
            for {
                let i := 1
            } lt(i, DILUTED_N_BITS) {
                i := add(i, 1)
            } {
                x := addmod(x, diffX, K_MODULUS)
                diffX := mulmod(diffX, diffMultiplier, K_MODULUS)
                // To save multiplications, store intermediate values.
                let x_p := mulmod(x, p, K_MODULUS)
                let y := add(p, mulmod(z, x_p, K_MODULUS))
                q := addmod(
                add(mulmod(q, y, K_MODULUS), mulmod(x, x_p, K_MODULUS)),
                    q,
                    K_MODULUS
                )
                p := mulmod(p, y, K_MODULUS)
            }
            res := addmod(p, mulmod(q, alpha, K_MODULUS), K_MODULUS)
        }
    }
}
// ---------- End of auto-generated code. ----------
