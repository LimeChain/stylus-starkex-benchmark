extern crate alloc;
use alloy_sol_types::sol;
use stylus_sdk:: prelude::*;

sol! {
    error GenericError();
}

#[derive(SolidityError)]
pub enum StarkError {
    GenericError(GenericError),
}

#[macro_export]
macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err(format!($msg).as_bytes().to_vec());
        }
    };
}
