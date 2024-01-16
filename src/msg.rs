use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128};

use crate::state::BookEntry;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // pub count: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateBookEntry { cw20_address: Addr, payment_cw20_address: Addr, amount: Uint128, price: Uint128 },
    UpdateBookEntry { id: u64, cw20_address: Addr, payment_cw20_address: Addr, amount: Uint128, price: Uint128 },
    DeleteBookEntry { id: u64 },
    TransferFrom { cw20_address: String, sender: String, recipient: String, amount: Uint128 },
    Buy { id: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BookEntry { id: u64 },
    BookList {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BookListResponse {
    pub book_entrys: Vec<BookEntry>,
}
