use stylus_sdk::prelude::sol_interface;

sol_interface! {
    interface IMerkleStatement {
        function is_valid(bytes32 statement) external view returns(bool);
    }

    interface IFriStatement {
        function is_valid(bytes32 statement) external view returns(bool);
    }

    interface IConstraintPoly {
        function compute(uint256[] memory calldata) external view returns(uint256);
    }

    // interface ICpuOods {
    //     function compute(uint256[] memory ctx) external view returns(uint256[] memory);
    // }

    interface IConstraint {
        function compute(uint256 value) external view returns(uint256);
    }

    interface IInitVerifier {
        function initVerifierParams(uint256[] memory public_input, uint256[] memory proof_params) external view returns(uint256[] memory ctx, uint256[] memory fri_step_sizes);
    }

    interface IFriStatementVerifier {
        function verifyFri(uint256[] memory proof, uint256[] memory ctx, uint256[] memory fri_step_sizes) external view;
    }
}
