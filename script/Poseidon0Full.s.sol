// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.6.12;

import "forge-std/Script.sol";
import {PoseidonPoseidonFullRoundKey0Column} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonFullRoundKey0Column.sol";

contract Poseidon0Full is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        PoseidonPoseidonFullRoundKey0Column poseidonColumn = new PoseidonPoseidonFullRoundKey0Column();
        console.log(
            "PoseidonPoseidonFullRoundKey0Column deployed at:",
            address(poseidonColumn)
        );

        vm.stopBroadcast();
    }
}
