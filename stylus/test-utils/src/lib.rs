#[macro_export]
macro_rules! try_execute {
    ($func_call:expr) => {
        match $func_call {
            Ok(data) => data,
            Err(data) => panic!("{:?}", String::from_utf8(data).unwrap()),
        };
    };
}
