// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.6.12;

import "forge-std/Script.sol";
import {PedersenHashPointsXColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PedersenHashPointsXColumn.sol";
import {PedersenHashPointsYColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PedersenHashPointsYColumn.sol";
import {EcdsaPointsXColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/EcdsaPointsXColumn.sol";
import {EcdsaPointsYColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/EcdsaPointsYColumn.sol";

contract PeriodicColumns is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();
        PedersenHashPointsXColumn xColumn = new PedersenHashPointsXColumn();
        PedersenHashPointsYColumn yColumn = new PedersenHashPointsYColumn();
        console.log("PedersenHashPointsXColumn deployed at:", address(xColumn));
        console.log("PedersenHashPointsYColumn deployed at:", address(yColumn));
        EcdsaPointsXColumn ecdsaXColumn = new EcdsaPointsXColumn();
        console.log("EcdsaPointsXColumn deployed at:", address(ecdsaXColumn));
        EcdsaPointsYColumn ecdsaYColumn = new EcdsaPointsYColumn();
        console.log("EcdsaPointsYColumn deployed at:", address(ecdsaYColumn));
        vm.stopBroadcast();
    }
}
