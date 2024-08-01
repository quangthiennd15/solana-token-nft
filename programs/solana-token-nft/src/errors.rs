use anchor_lang::prelude::*;

#[error_code]
pub enum MarketPlaceError {
    #[msg("Item is canceled !")]
    ItemCanceledInvalid,

    #[msg("Insufficient balance !")]
    InsufficientBalance,
}
#[error_code]
pub enum SigError {
    #[msg("Signature verification failed.")]
    SigVerificationFailed,
}

