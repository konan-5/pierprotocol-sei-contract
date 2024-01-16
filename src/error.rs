use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Insufficient allowance - amount: {amount}, allowance: {allowance}")]
    InsufficientAllowance {
        amount: Uint128,
        allowance: Uint128,
    },

    #[error("Insufficient amount")]
    InsufficientAmount {},

    #[error("Order adress and payment address must not be equal")]
    NotValidPaymentAddress{},
}
