use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WaitingList {
    pub id: u64,
    pub owner: Addr,
    pub contract: Addr,
    pub amount: u128,
    pub price: u128,
}

pub const STATE: Item<State> = Item::new("state");
pub const WAITINGLIST_COUNTER: Item<u64> = Item::new("waitinglist_counter");
pub const WAITINGLIST: Map<u64, WaitingList> = Map::new("waitinglist");
