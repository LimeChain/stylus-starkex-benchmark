// SPDX-License-Identifier: MIT
pragma solidity ^0.6.12;

import "forge-std/Script.sol";
import "../evm-verifier/solidity/contracts/cpu/MemoryPageFactRegistry.sol";

contract DeployMemoryPageFactRegistry is Script {
    function run() public {
        vm.startBroadcast();
        MemoryPageFactRegistry registry = new MemoryPageFactRegistry();
        console.log("MemoryPageFactRegistry deployed at:", address(registry));
        vm.stopBroadcast();
    }
}
