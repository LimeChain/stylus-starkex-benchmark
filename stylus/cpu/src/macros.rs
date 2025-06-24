#[macro_export]
macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err(VerifierError::InvalidProof(InvalidProof {
                reason: $msg.to_string(),
            }));
        }
    };
}
