// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.6.12;

import "forge-std/Script.sol";
import {PedersenHashPointsXColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PedersenHashPointsXColumn.sol";
import {PedersenHashPointsYColumn} from "../evm-verifier/solidity/contracts/cpu/periodic_columns/PedersenHashPointsYColumn.sol";

contract PedersenHashPointsXColumnDeploy is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();
        new PedersenHashPointsXColumn();
        new PedersenHashPointsYColumn();
        vm.stopBroadcast();
    }
}
