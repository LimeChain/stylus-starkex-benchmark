// SPDX-License-Identifier: MIT
pragma solidity ^0.6.12;
pragma experimental ABIEncoderV2;

import "forge-std/Test.sol";
import "../evm-verifier/solidity/contracts/cpu/periodic_columns/PedersenHashPointsXColumn.sol";
import "../evm-verifier/solidity/contracts/cpu/periodic_columns/PedersenHashPointsYColumn.sol";
import "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonFullRoundKey0Column.sol";
import "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonFullRoundKey1Column.sol";
import "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonFullRoundKey2Column.sol";
import "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonPartialRoundKey0Column.sol";
import "../evm-verifier/solidity/contracts/cpu/periodic_columns/PoseidonPoseidonPartialRoundKey1Column.sol";

contract PeriodicColumnsTest is Test {
    PedersenHashPointsXColumn xColumn;
    PedersenHashPointsYColumn yColumn;

    PoseidonPoseidonFullRoundKey0Column poseidonColumn;
    PoseidonPoseidonFullRoundKey1Column poseidonColumn1;
    PoseidonPoseidonFullRoundKey2Column poseidonColumn2;
    PoseidonPoseidonPartialRoundKey0Column poseidonColumn0;
    PoseidonPoseidonPartialRoundKey1Column poseidonColumn11;

    uint256 pederson_input =
        2502371038239847331946845555940821891939660827069539886818086403686260021246;

    uint256 poseidon_input =
        513761785516736576210258345954495650460389361631034617172115002511570125974;

    function setUp() public {
        xColumn = new PedersenHashPointsXColumn();
        yColumn = new PedersenHashPointsYColumn();

        poseidonColumn = new PoseidonPoseidonFullRoundKey0Column();
        poseidonColumn1 = new PoseidonPoseidonFullRoundKey1Column();
        poseidonColumn2 = new PoseidonPoseidonFullRoundKey2Column();
        poseidonColumn0 = new PoseidonPoseidonPartialRoundKey0Column();
        poseidonColumn11 = new PoseidonPoseidonPartialRoundKey1Column();
    }

    function testPedersenHashPointsColumnCompute() public view {
        uint256 expectedResultX = 2476435194882991550378205418214791165604712474576866766823810310226558062065;
        uint256 expectedResultY = 1444533035788560090889078696321009507857064390212204404518903797387225515076;
        uint256 resultX = xColumn.compute(pederson_input);
        uint256 resultY = yColumn.compute(pederson_input);

        console.log("resultX", resultX);
        console.log("expectedResultX", expectedResultX);
        console.log("resultY", resultY);
        console.log("expectedResultY", expectedResultY);

        assertEq(resultX, expectedResultX);
        assertEq(resultY, expectedResultY);
    }

    function testPedersonFrk0ColumnCompute() public view {
        uint256 expectedResult = 1747952454919021766681010400995206390562374609324430906386085649753967957996;
        uint256 result = poseidonColumn.compute(poseidon_input);
        assertEq(result, expectedResult);
    }

    function testPoseidonPoseidonColumnCompute() public view {
        // full round key
        uint256 full_round_key0_column_expectedResult = 1747952454919021766681010400995206390562374609324430906386085649753967957996;
        uint256 full_round_key0_column_result = poseidonColumn.compute(
            poseidon_input
        );

        uint256 full_round_key1_column_expectedResult = 1664257228653772301912891197477956780973260593455413394763471271235501957228;

        uint256 full_round_key1_column_result = poseidonColumn1.compute(
            poseidon_input
        );

        uint256 full_round_key2_column_expectedResult = 1938976483485279484363264204509611131731729867572976629648616677903267220493;

        uint256 full_round_key2_column_result = poseidonColumn2.compute(
            poseidon_input
        );

        assertEq(
            full_round_key0_column_result,
            full_round_key0_column_expectedResult
        );
        assertEq(
            full_round_key1_column_result,
            full_round_key1_column_expectedResult
        );
        assertEq(
            full_round_key2_column_result,
            full_round_key2_column_expectedResult
        );

        // Partial round key
        uint256 partial_round_key0_column_expectedResult = 1499007735260395255086346814066654016187033964386904667040298584658325794077;
        uint256 partial_round_key0_column_result = poseidonColumn0.compute(
            poseidon_input
        );

        assertEq(
            partial_round_key0_column_result,
            partial_round_key0_column_expectedResult
        );

        uint256 partial_round_key1_column_expectedResult = 2486570557154671379335084513491649861794821253711847039152551529444239535533;
        uint256 partial_round_key1_column_result = poseidonColumn11.compute(
            poseidon_input
        );

        assertEq(
            partial_round_key1_column_result,
            partial_round_key1_column_expectedResult
        );
    }
}
