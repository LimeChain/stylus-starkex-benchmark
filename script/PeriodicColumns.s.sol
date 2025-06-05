// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.6.12;

import "forge-std/Script.sol";
import {PedersenHashPointsXColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PedersenHashPointsXColumn.sol";
import {PedersenHashPointsYColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PedersenHashPointsYColumn.sol";
// import {EcdsaPointsXColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/EcdsaPointsXColumn.sol";
// import {EcdsaPointsYColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/EcdsaPointsYColumn.sol";
import {PoseidonPoseidonFullRoundKey0Column} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonFullRoundKey0Column.sol";
import {PoseidonPoseidonFullRoundKey1Column} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonFullRoundKey1Column.sol";
import {PoseidonPoseidonFullRoundKey2Column} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonFullRoundKey2Column.sol";
import {PoseidonPoseidonPartialRoundKey0Column} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonPartialRoundKey0Column.sol";
import {PoseidonPoseidonPartialRoundKey1Column} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonPartialRoundKey1Column.sol";

contract PeriodicColumns is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();
        PedersenHashPointsXColumn xColumn = new PedersenHashPointsXColumn();
        PedersenHashPointsYColumn yColumn = new PedersenHashPointsYColumn();
        console.log("PedersenHashPointsXColumn deployed at:", address(xColumn));
        console.log("PedersenHashPointsYColumn deployed at:", address(yColumn));

        PoseidonPoseidonFullRoundKey0Column poseidonColumn = new PoseidonPoseidonFullRoundKey0Column();
        console.log(
            "PoseidonPoseidonFullRoundKey0Column deployed at:",
            address(poseidonColumn)
        );

        PoseidonPoseidonFullRoundKey1Column poseidonColumn1 = new PoseidonPoseidonFullRoundKey1Column();
        console.log(
            "PoseidonPoseidonFullRoundKey1Column deployed at:",
            address(poseidonColumn1)
        );

        PoseidonPoseidonFullRoundKey2Column poseidonColumn2 = new PoseidonPoseidonFullRoundKey2Column();
        console.log(
            "PoseidonPoseidonFullRoundKey2Column deployed at:",
            address(poseidonColumn2)
        );

        PoseidonPoseidonPartialRoundKey0Column poseidonColumn0 = new PoseidonPoseidonPartialRoundKey0Column();
        console.log(
            "PoseidonPoseidonPartialRoundKey0Column deployed at:",
            address(poseidonColumn0)
        );

        PoseidonPoseidonPartialRoundKey1Column poseidonColumn11 = new PoseidonPoseidonPartialRoundKey1Column();
        console.log(
            "PoseidonPoseidonPartialRoundKey1Column deployed at:",
            address(poseidonColumn11)
        );

        vm.stopBroadcast();
    }
}
