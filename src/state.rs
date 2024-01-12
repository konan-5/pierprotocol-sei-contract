use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: Uint128,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BookEntry {
    pub id: u64,
    pub owner: Addr,
    pub contract: Addr,
    pub amount: Uint128,
    pub price: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
pub const BOOK_ENTRY_SEQ: Item<u64> = Item::new("book_entry_seq");
pub const BOOK_LIST: Map<u64, BookEntry> = Map::new("book_list");
