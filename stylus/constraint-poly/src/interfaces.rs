use stylus_sdk::prelude::sol_interface;

sol_interface! {

    interface IConstraintPolyPreparer {
        function compute(uint256[] memory calldata) external view returns(uint256[] memory);
    }

    interface IConstraintPolyFinalizer {
        function compute(uint256[] memory calldata) external view returns(uint256);
    }
}
