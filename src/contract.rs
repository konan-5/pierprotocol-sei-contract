#[cfg(not(feature = "library"))]
use cosmwasm_std::{ entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Addr, Uint128, CosmosMsg, WasmMsg, QueryRequest, WasmQuery };
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw20::{ Cw20ExecuteMsg, Cw20QueryMsg, AllowanceResponse };

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, BookListResponse};
use crate::state::{BookEntry, BOOK_LIST, BOOK_ENTRY_SEQ};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:pierprotocol-sei";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    BOOK_ENTRY_SEQ.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateBookEntry { cw20_address, payment_cw20_address, amount, price } => execute_create_book_entry(deps, env, info, cw20_address, payment_cw20_address, amount, price),
        ExecuteMsg::UpdateBookEntry { id, cw20_address, payment_cw20_address, amount, price } => execute_update_book_entry(deps, env, info, id, cw20_address, payment_cw20_address, amount, price),
        ExecuteMsg::DeleteBookEntry { id } => execute_delete_book_entry( deps, info, id ),
        ExecuteMsg::Buy { id } => execute_buy( deps, env, info, id ),
    }
}

pub fn execute_buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let book_entry = BOOK_LIST.load(deps.storage, id)?;
    let allowance = get_allowance(&deps, &env, &info, &book_entry.payment_cw20_address)?;

    if allowance < book_entry.price {
        return Err(ContractError::InsufficientAmount {});
    };

    // swap token
    let msg = Cw20ExecuteMsg::TransferFrom {
        owner: book_entry.owner.to_string(),
        recipient: info.sender.to_string(),
        amount: book_entry.amount,
    };

    let cosmos_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: book_entry.cw20_address.to_string(),
        msg: to_json_binary(&msg)?,
        funds: vec![],
    });

    let payment_msg = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.to_string(),
        recipient: book_entry.owner.to_string(),
        amount: book_entry.price,
    };

    let payment_cosmos_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: book_entry.payment_cw20_address.to_string(),
        msg: to_json_binary(&payment_msg)?,
        funds: vec![],
    });

    // remove book
    BOOK_LIST.remove(deps.storage, id);

    Ok(Response::new()
        .add_message(payment_cosmos_msg)
        .add_message(cosmos_msg)
        .add_attribute("method", "execute_buy")
        .add_attribute("swaped_book_entry_id", id.to_string()))
}

pub fn execute_create_book_entry(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_address: Addr,
    payment_cw20_address: Addr,
    amount: Uint128,
    price: Uint128,
) -> Result<Response, ContractError> {
    if cw20_address.to_string() == payment_cw20_address.to_string() {
        return Err(ContractError::NotValidPaymentAddress {});
    };

    if amount <= Uint128::new(0) {
        return Err(ContractError::InsufficientAmount {});
    };

    let allowance = get_allowance(&deps, &env, &info, &cw20_address)?;

    if amount > allowance {
        return Err(ContractError::InsufficientAllowance {amount, allowance});
    };

    let id = BOOK_ENTRY_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id + 1))?;

    let sender = info.sender;
    let book_entry = BookEntry {
        id,
        owner: sender,
        cw20_address,
        payment_cw20_address,
        amount,
        price,
    };

    BOOK_LIST.save(deps.storage, id, &book_entry)?;

    Ok(Response::new()
        .add_attribute("method", "execute_create_book_entry")
        .add_attribute("new_book_entry", id.to_string()))
}

fn get_allowance(deps: &DepsMut, env: &Env, info: &MessageInfo, cw20_address: &Addr) -> Result<Uint128, ContractError> {
    let allowance_query = Cw20QueryMsg::Allowance {
        owner: info.sender.to_string(),
        spender: env.contract.address.to_string(),
    };
    let wasm_query = WasmQuery::Smart {
        contract_addr: cw20_address.to_string(),
        msg: to_json_binary(&allowance_query)?,
    }.into();
    let allowance_response: AllowanceResponse = deps.querier.query(&QueryRequest::Wasm(wasm_query))?;

    Ok(allowance_response.allowance)
}

pub fn execute_update_book_entry(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    cw20_address: Addr,
    payment_cw20_address: Addr,
    amount: Uint128,
    price: Uint128,
) -> Result<Response, ContractError> {
    if cw20_address.to_string() == payment_cw20_address.to_string() {
        return Err(ContractError::NotValidPaymentAddress {});
    };

    if amount <= Uint128::new(0) {
        return Err(ContractError::InsufficientAmount {});
    };

    let allowance = get_allowance(&deps, &env, &info, &cw20_address)?;
    if amount > allowance {
        return Err(ContractError::InsufficientAllowance {amount, allowance});
    };

    let sender = info.sender;
    let book_entry = BOOK_LIST.load(deps.storage, id)?;
    if book_entry.owner != sender {
        return Err(ContractError::Unauthorized {});
    }
    let updated_book_entry = BookEntry {
        id,
        owner: sender,
        cw20_address,
        payment_cw20_address,
        amount,
        price,
    };
    BOOK_LIST.save(deps.storage, id, &updated_book_entry)?;
    Ok(Response::new()
        .add_attribute("method", "execute_update_book_entry")
        .add_attribute("updated_book_entry_id", id.to_string()))
}

pub fn execute_delete_book_entry(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let book_entry = BOOK_LIST.load(deps.storage, id)?;
    if book_entry.owner != sender {
        return Err(ContractError::Unauthorized {});
    }
    BOOK_LIST.remove(deps.storage, id);
    Ok(Response::new()
        .add_attribute("method", "execute_delete_book_entry")
        .add_attribute("deleted_book_entry_id", id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BookEntry { id } => to_json_binary(&query_book_entry(deps, id)?),
        QueryMsg::BookList { start_after, limit } => to_json_binary(&query_book_list(deps, start_after, limit)?),
    }
}

fn query_book_entry(deps: Deps, id: u64) -> StdResult<BookEntry> {
    let book_entry = BOOK_LIST.load(deps.storage, id)?;
    Ok(BookEntry {
        id: id,
        owner: book_entry.owner,
        cw20_address: book_entry.cw20_address,
        payment_cw20_address: book_entry.payment_cw20_address,
        amount: book_entry.amount,
        price: book_entry.price,
    })
}

// Limits for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
fn query_book_list(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<BookListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let book_entrys: StdResult<Vec<_>> = BOOK_LIST
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();
    let result = BookListResponse {
        book_entrys: book_entrys?.into_iter().map(|l| l.1).collect(),
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
