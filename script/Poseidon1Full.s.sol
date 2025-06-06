// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.6.12;

import "forge-std/Script.sol";
import {PoseidonPoseidonFullRoundKey1Column} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonFullRoundKey1Column.sol";

contract Poseidon0Full is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        PoseidonPoseidonFullRoundKey1Column poseidonColumn = new PoseidonPoseidonFullRoundKey1Column();
        console.log(
            "PoseidonPoseidonFullRoundKey1Column deployed at:",
            address(poseidonColumn)
        );

        vm.stopBroadcast();
    }
}
