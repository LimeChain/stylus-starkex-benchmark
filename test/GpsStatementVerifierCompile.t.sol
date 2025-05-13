// SPDX-License-Identifier: MIT
pragma solidity ^0.6.12;

import "forge-std/Test.sol";
import "../evm-verifier/solidity/contracts/gps/GpsStatementVerifier.sol";

contract GpsStatementVerifierCompile {
    // No tests—compiling is the test.
    function testCompile() public pure {
        // empty body – the fact this runs means compilation succeeded
        require(true);
    }
}
