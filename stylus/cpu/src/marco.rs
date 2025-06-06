macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($msg.to_vec());
        }
    };
}
