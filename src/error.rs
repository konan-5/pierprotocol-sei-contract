use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Insufficient allowance")]
    InsufficientAllowance {},

    #[error("Insufficient amount")]
    InsufficientAmount {},

    #[error("Order adress and payment address must not be equal")]
    NotValidPaymentAddress{},
}
