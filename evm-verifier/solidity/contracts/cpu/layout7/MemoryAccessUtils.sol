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
// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.6.12;

import "./MemoryMap.sol";

contract MemoryAccessUtils is MemoryMap {
    function getPtr(uint256[] memory ctx, uint256 offset) internal pure returns (uint256) {
        uint256 ctxPtr;
        require(offset < MM_CONTEXT_SIZE, "Overflow protection failed");
        assembly {
            ctxPtr := add(ctx, 0x20)
        }
        return ctxPtr + offset * 0x20;
    }

    function getProofPtr(uint256[] memory proof) internal pure returns (uint256) {
        uint256 proofPtr;
        assembly {
            proofPtr := proof
        }
        return proofPtr;
    }

    function getChannelPtr(uint256[] memory ctx) internal pure returns (uint256) {
        uint256 ctxPtr;
        assembly {
            ctxPtr := add(ctx, 0x20)
        }
        return ctxPtr + MM_CHANNEL * 0x20;
    }

    function getQueries(uint256[] memory ctx) internal pure returns (uint256[] memory) {
        uint256[] memory queries;
        // Dynamic array holds length followed by values.
        uint256 offset = 0x20 + 0x20 * MM_N_UNIQUE_QUERIES;
        assembly {
            queries := add(ctx, offset)
        }
        return queries;
    }

    function getMerkleQueuePtr(uint256[] memory ctx) internal pure returns (uint256) {
        return getPtr(ctx, MM_MERKLE_QUEUE);
    }

    function getFriStepSizes(uint256[] memory ctx)
        internal
        pure
        returns (uint256[] memory friStepSizes)
    {
        uint256 val;
        uint256 friStepSizesPtr = getPtr(ctx, MM_FRI_STEP_SIZES_PTR);
        assembly {
            friStepSizes := mload(friStepSizesPtr)
            val := mload(friStepSizesPtr)
        }
    }
}
