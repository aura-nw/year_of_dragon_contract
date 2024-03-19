#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_json_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;

use nois::{NoisCallback, ProxyExecuteMsg};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, CONFIG, DRAND_GENESIS, DRAND_ROUND_LENGTH, DRAND_ROUND_WITH_FORGE_HASH, RANDOM_JOBS, RANDOM_SEED},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wheel-of-fortune";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        contract_owner: info.sender.clone(),
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
        ExecuteMsg::ForgeGem { request_forge_hash } => {
            execute_forge_gem(deps, env, info, request_forge_hash)
        }
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
    // Only contract owner can forge gem
    ensure_eq!(
        info.sender,
        config.contract_owner,
        ContractError::Unauthorized {}
    );
    // Load the nois_proxy
    let nois_proxy = config.nois_proxy;

    let funds = info.funds;

    let mut res = Response::new();

    let drand_round: u64;

    let after = env.block.time;

    // Make randomness request message to NOIS proxy contract
    let msg_make_randomess = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nois_proxy.into(),
        msg: to_json_binary(&ProxyExecuteMsg::GetRandomnessAfter {
            job_id: request_forge_hash.clone(),
            after,
        })?,
        funds,
    });

    // Losely ported from https://github.com/drand/drand/blob/eb36ba81e3f28c966f95bcd602f60e7ff8ef4c35/chain/time.go#L49-L63
    if after < DRAND_GENESIS {
        drand_round = 1
    } else {
        let from_genesis = after.nanos() - DRAND_GENESIS.nanos();
        let periods_since_genesis = from_genesis / DRAND_ROUND_LENGTH;
        let next_period_index = periods_since_genesis + 1;
        drand_round = next_period_index + 1 // Convert 0-based counting to 1-based counting
    }

    res = res.add_message(msg_make_randomess);

    RANDOM_JOBS.save(
        deps.storage,
        request_forge_hash.clone(),
        &"waiting...".to_string(),
    )?;

    DRAND_ROUND_WITH_FORGE_HASH.save(
        deps.storage,
        request_forge_hash.clone(),
        &drand_round.to_string(),
    )?;

    Ok(res.add_attribute("action", "forge_gem")
        .add_attribute("request_forge_hash", request_forge_hash)
        .add_attribute("after", after.seconds().to_string())
        .add_attribute("drand_round", drand_round.to_string()))
}

pub fn nois_receive(
    deps: DepsMut,
    _env: Env,
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
    // let randomness: [u8; 32] = callback
    //     .randomness
    //     .to_array()
    //     .map_err(|_| ContractError::InvalidRandomness {})?;

    // Convert the random seed to string
    let randomness_string: String = callback.randomness.to_hex();

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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::RandomSeed {} => to_json_binary(&query_random_seed(deps)?),
        QueryMsg::RandomSeedFromRequestForgeHash { request_forge_hash } => to_json_binary(
            &query_random_seed_from_request_forge_hash(deps, request_forge_hash)?,
        ),
        QueryMsg::DrandRoundWithForgeHash { request_forge_hash } => to_json_binary(
            &query_drand_round_with_forge_hash(deps, request_forge_hash)?,
        ),
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

fn query_random_seed_from_request_forge_hash(
    deps: Deps,
    request_forge_hash: String,
) -> StdResult<String> {
    let random_job = RANDOM_JOBS.load(deps.storage, request_forge_hash)?;
    Ok(random_job)
}

fn query_drand_round_with_forge_hash(
    deps: Deps,
    request_forge_hash: String,
) -> StdResult<String> {
    let drand_round = DRAND_ROUND_WITH_FORGE_HASH.load(deps.storage, request_forge_hash)?;
    Ok(drand_round)
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
mod test_nois_receive {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, HexBinary, Timestamp,
    };
    use nois::NoisCallback;

    use crate::state::{Config, CONFIG};

    #[test]
    fn test_nois_receive() {
        let mut deps = mock_dependencies();

        let env = mock_env();

        let sender = "nois_proxy".to_string();
        let funds = vec![];

        let info = mock_info(&sender, &funds);
        let config = {
            Config {
                nois_proxy: Addr::unchecked("nois_proxy"),
                contract_owner: Addr::unchecked("contract_owner"),
            }
        };
        CONFIG.save(&mut deps.storage, &config).unwrap();

        let job_id = "job_id".to_string();
        let randomness =
            "46FAF1CD4845AB7C5A9DAA7D272259682BF84176A2658DE67CB1317A22134973".to_string();
        let callback = NoisCallback {
            job_id: job_id.clone(),
            published: Timestamp::from_seconds(1682086395),
            randomness: HexBinary::from_hex(&randomness).unwrap(),
        };

        let res = super::nois_receive(deps.as_mut(), env, info, callback).unwrap();

        assert_eq!(res.attributes.len(), 2);
    }
}
