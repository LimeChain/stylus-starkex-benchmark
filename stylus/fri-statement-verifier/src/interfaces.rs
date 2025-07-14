use stylus_sdk::prelude::sol_interface;

sol_interface! {
    interface ICpuOods {
        function compute(uint256[] memory ctx) external view returns(uint256[] memory);
    }
}
