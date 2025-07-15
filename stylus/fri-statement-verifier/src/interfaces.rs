use stylus_sdk::prelude::sol_interface;

sol_interface! {
    interface IFriStatementVerifier {
        function isValid(bytes32 fact) external view returns (bool);
    }

    interface IMerkleStatementVerifier {
        function isValid(bytes32 fact) external view returns (bool);
    }

    interface ICpuOods {
        function compute(uint256[] memory ctx) external view returns(uint256[] memory);
    }
}
