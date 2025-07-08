use stylus_sdk::prelude::sol_interface;

sol_interface! {
    interface IMerkleStatement {
        function is_valid(bytes32 statement) external view returns(bool);
    }

    interface IFriStatement {
        function is_valid(bytes32 statement) external view returns(bool);
    }

    interface IConstraintPoly {
        function compute(bytes memory calldata) external view returns(uint256);
    }

    interface ICpuOods {
        function compute(uint256[] memory ctx) external view returns(uint256[] memory);
    }

    interface IConstraint {
        function compute(uint256 value) external view returns(uint256);
    }
}
