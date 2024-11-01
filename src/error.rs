use thiserror::Error;

#[derive(Error, Debug)]
pub enum Transaction {
    #[error("Transaction ID {0} not found for client {1}")]
    NotFound(u32, u16),

    #[error("Insufficient funds for client {0}")]
    InsufficientFunds(u16),

    #[error("Account {0} is locked")]
    AccountLocked(u16),

    #[error("Invalid amount for transaction ID {0}")]
    InvalidAmount(u32),

    #[error("Transaction ID {0} is already under dispute")]
    AlreadyDisputed(u32),

    #[error("Transaction ID {0} is not under dispute")]
    NotUnderDispute(u32),

    #[error("Cannot dispute transaction ID {0} as it is not a deposit")]
    InvalidDispute(u32),

    #[error("Cannot chargeback transaction ID {0} as it is not a deposit")]
    InvalidChargeback(u32),
}
