#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, WaitingList, WAITINGLIST, WAITINGLIST_COUNTER};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:pierprotocol-sei";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
        ExecuteMsg::CreateWatingItem { contract, amount, price } => execute_create_waiting_item(deps, info, contract, amount, price),
        ExecuteMsg::UpdateWatingItem { id, contract, amount, price } => execute_update_waiting_item(deps, info, id, contract, amount, price),
        ExecuteMsg::DeleteWatingItem { id } => execute_delete_waiting_item( deps, info, id ),
    }
}

pub fn execute_create_waiting_item(
    deps: DepsMut, 
    info: MessageInfo, 
    contract: Addr, 
    amount: u128, 
    price: u128,
) -> Result<Response, ContractError> {
    let id = WAITINGLIST_COUNTER.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id + 1))?;
    
    let sender = info.sender;
    let waiting_item = WaitingList {
        id,
        owner: sender,
        contract,
        amount,
        price,
    };

    WAITINGLIST.save(deps.storage, id, &waiting_item)?;

    Ok(Response::new()
        .add_attribute("method", "execute_create_waiting_item")
        .add_attribute("new_waiting_item_id", id.to_string()))
}

pub fn execute_update_waiting_item(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    contract: Addr,
    amount: u128,
    price: u128,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let waiting_item = WAITINGLIST.load(deps.storage, id)?;
    if waiting_item.owner != sender {
        return Err(ContractError::Unauthorized {});
    }
    let update_waiting_item = WaitingList {
        id,
        owner: sender,
        contract,
        amount,
        price,
    };
    WAITINGLIST.save(deps.storage, id, &update_waiting_item)?;
    Ok(Response::new()
        .add_attribute("method", "execute_update_waiting_item")
        .add_attribute("updated_waiting_item_id", id.to_string()))
}

pub fn execute_delete_waiting_item(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let waiting_item = WAITINGLIST.load(deps.storage, id)?;
    if waiting_item.owner != sender {
        return Err(ContractError::Unauthorized {});
    }
    WAITINGLIST.remove(deps.storage, id);
    Ok(Response::new()
        .add_attribute("method", "execute_delete_waiting_item")
        .add_attribute("deleted_waiting_item_id", id.to_string()))
}

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
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

    #[test]
    fn add_to_waiting_list() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        // let msg = InstantiateMsg { count: 17 };
        let info = mock_info("sender", &coins(2, "token"));

        let contract = Addr::unchecked("contract_addr");
        let amount: u128 = 1000;
        let price: u128 = 50;
        // let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::AddWatingList { contract, amount, price };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
