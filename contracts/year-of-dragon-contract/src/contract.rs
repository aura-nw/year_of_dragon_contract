use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, has_coins, to_binary, to_json_binary, wasm_execute, Addr, Api, BalanceResponse, BankMsg, BankQuery, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env, HexBinary, MessageInfo, Order, QueryRequest, Response, StdResult, Storage, Timestamp, Uint128, WasmMsg, WasmQuery
};
use cw2::set_contract_version;

use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, NftInfoResponse};
use cw721_base::ExecuteMsg as Cw721BaseExecuteMsg;

use cw20::Cw20ExecuteMsg;
use nois::{randomness_from_str, select_from_weighted, sub_randomness_with_key, NoisCallback, ProxyExecuteMsg};

use crate::{error::ContractError, msg::{ExecuteMsg, InstantiateMsg, QueryMsg}, state::{AuragonURI, Config, GemInfo, GemMetadata, Metadata, RandomJob, RequestForgeGemInfo, Trait, UserFinalGemInfo, UserInfo, CONFIG, RANDOM_JOBS, RANDOM_SEED}};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wheel-of-fortune";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_TEXT_LENGTH: usize = 253;
const MAX_VEC_ITEM: usize = 65536;
const MAX_SPINS_PER_TURN: u32 = 10;
const DEFAULT_ACTIVATE: bool = false;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let nois_proxy = addr_validate(deps.api, &msg.nois_proxy)?;

    let config = Config {
        nois_proxy,
    };

    CONFIG.save(deps.storage, &config)?;

    RANDOM_SEED.save(deps.storage, &msg.random_seed)?;

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
        ExecuteMsg::ForgeGem { request_forge_hash }
            => execute_forge_gem(deps, env, info, request_forge_hash),
        //nois callback
        ExecuteMsg::NoisReceive { callback } => nois_receive(deps, env, info, callback),
    }
}

pub fn execute_forge_gem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    request_forge_hash: String,
) -> Result<Response, ContractError> {
    // Load the config
    let config = CONFIG.load(deps.storage)?;
    // Load the nois_proxy
    let nois_proxy = config.nois_proxy;

    let funds = info.funds;

    let mut res = Response::new();

    // Make randomness request message to NOIS proxy contract
    let msg_make_randomess = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nois_proxy.into(),
        msg: to_json_binary(&ProxyExecuteMsg::GetNextRandomness {
            job_id: request_forge_hash.clone(),
        })?,
        funds,
    });

    res = res.add_message(msg_make_randomess);

    RANDOM_JOBS.save(deps.storage, request_forge_hash.clone(), &"waiting...".to_string())?;
    Ok(res)
}

pub fn nois_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    callback: NoisCallback,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    ensure_eq!(
        info.sender,
        config.nois_proxy,
        ContractError::Unauthorized {}
    );
    let res = Response::new();
    let job_id = callback.job_id;
    let randomness: [u8; 32] = callback
        .randomness
        .to_array()
        .map_err(|_| ContractError::InvalidRandomness {})?;

    // Convert the random seed to string
    let randomness_string: String = callback.randomness.to_string();

    // update random seed
    RANDOM_SEED.save(deps.storage, &randomness_string)?;

    // update random job
    RANDOM_JOBS.save(deps.storage, job_id.clone(), &randomness_string)?;

    Ok(res
        .add_attribute("action", "nois_receive")
        .add_attribute("job_id", job_id))
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::RandomSeed {} => to_json_binary(&query_random_seed(deps)?),
        QueryMsg::RandomSeedFromRequestForgeHash { request_forge_hash } => to_json_binary(&query_random_seed_from_request_forge_hash(deps, request_forge_hash)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_random_seed(deps: Deps) -> StdResult<String> {
    let random_seed = RANDOM_SEED.load(deps.storage)?;
    // Convert the random seed to string
    Ok(random_seed)
}

fn query_random_seed_from_request_forge_hash(deps: Deps, request_forge_hash: String) -> StdResult<String> {
    let random_job = RANDOM_JOBS.load(deps.storage, request_forge_hash)?;
    Ok(random_job)
}

/// validate string if it is valid bench32 string addresss
fn addr_validate(api: &dyn Api, addr: &str) -> Result<Addr, ContractError> {
    let addr = api
        .addr_validate(addr)
        .map_err(|_| ContractError::InvalidAddress {})?;
    Ok(addr)
}

// Unit test for select_gem_rewards
#[cfg(test)]
mod test_select_gem_rewards {
    use cosmwasm_std::{testing::mock_dependencies, Addr, Timestamp};

    use crate::{state::{AuragonURI, Config, RandomJob, CONFIG, RANDOM_JOBS}};

    #[test]
    fn test_select_gem_rewards() {
        let mut deps = mock_dependencies();
        let config = {
            Config {
                nois_proxy: Addr::unchecked("nois_proxy"),
            }
        };
        CONFIG.save(&mut deps.storage, &config).unwrap();
        let auragon_gem_latest_token_id = 1;


        println!("AURA LATEST TOKEN: {:?}", auragon_gem_latest_token_id);
        // assert_eq!(res.unwrap().messages, vec![]);
    }
}
