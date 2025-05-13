// SPDX-License-Identifier: MIT
pragma solidity ^0.6.12;
pragma experimental ABIEncoderV2;

import "forge-std/Test.sol";
import "../evm-verifier/solidity/contracts/cpu/MemoryPageFactRegistry.sol";

// forge test --match-contract MemoryPageFactRegistryTest \
// 	--fork-url anvil -vvv
contract MemoryPageFactRegistryTest is Test {
    MemoryPageFactRegistry registry;

    function setUp() public {
        registry = new MemoryPageFactRegistry();
    }

    function testCompile() public pure {
        // empty body – the fact this runs means compilation succeeded
        require(true);
    }

    function testRegisterContinuousMemoryPage() public {
        // Starting address
        uint256 startAddr = 1000;

        // Create an array of values for consecutive addresses
        uint256[] memory values = new uint256[](5);
        values[0] = 100; // Value at address 1000
        values[1] = 200; // Value at address 1001
        values[2] = 300; // Value at address 1002
        values[3] = 400; // Value at address 1003
        values[4] = 500; // Value at address 1004

        // Same parameters as in the regular memory page test
        uint256 z = 5;
        uint256 alpha = 3;
        uint256 prime = 2 ** 251 + 17 * 2 ** 192 + 1; // Prime field used by StarkWare

        // Call the function
        (bytes32 factHash, , ) = registry.registerContinuousMemoryPage(
            startAddr,
            values,
            z,
            alpha,
            prime
        );

        // Verify the fact was registered
        assertTrue(
            registry.isValid(factHash),
            "Continuous memory page fact should be registered"
        );
    }

    function testRegisterRegularMemoryPage() public {
        uint256[] memory memoryPairs = new uint256[](4);
        // Address, value pairs
        memoryPairs[0] = 1;
        memoryPairs[1] = 100;
        memoryPairs[2] = 2;
        memoryPairs[3] = 200;

        uint256 z = 5;
        uint256 alpha = 3;
        uint256 prime = 2 ** 251 + 17 * 2 ** 192 + 1; // Prime field used by StarkWare

        (bytes32 factHash, uint256 memoryHash, uint256 prod) = registry
            .registerRegularMemoryPage(memoryPairs, z, alpha, prime);

        console.log("memoryHash:", memoryHash);
        console.log("prod:", prod);
        console.log("factHash:", vm.toString(factHash));
        console.log("isValid:", registry.isValid(factHash));

        assertTrue(registry.isValid(factHash), "Fact should be registered");
        assertEq(
            memoryHash,
            73303762061477191319875668523507331965327761895046903539761298990706739567530
        );
        assertEq(prod, 176712);
        assertEq(
            factHash,
            0xeb4573be19285f49cf74a74d3b35b14a8d601493ea9c2bf199eb34b4ebc0f5c7
        );
    }
}
