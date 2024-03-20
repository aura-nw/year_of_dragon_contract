#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_json_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, HexBinary,
    MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;

use nois::{select_from_weighted, sub_randomness_with_key, NoisCallback, ProxyExecuteMsg};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{
        Config, MigrateMsg, RandomJobs, RandomResponse, CONFIG, DRAND_GENESIS, DRAND_ROUND_LENGTH, DRAND_ROUND_WITH_HASH, JACKPOT_GEMS_WITH_CAMPAIGN_ID, RANDOM_JOBS, RANDOM_SEED
    },
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:year-of-dragon";
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
        contract_operator: Addr::unchecked(msg.operator),
    };

    CONFIG.save(deps.storage, &config)?;

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
        ExecuteMsg::SelectJackpotGems { campaign_id } => {
            execute_select_jackpot_gems(deps, env, info, campaign_id)
        }
        ExecuteMsg::UpdateConfig {
            nois_proxy,
            operator,
        } => execute_update_config(deps, info, nois_proxy, operator),
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
        config.contract_operator,
        ContractError::Unauthorized {}
    );
    // if request_forge_hash already exists in RANDOM_JOBS then return error
    if RANDOM_JOBS.may_load(deps.storage, request_forge_hash.clone())?.is_some() {
        return Err(ContractError::InvalidForgeHash {});
    }
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

    let random_jobs = RandomJobs {
        randomness: "waiting...".to_string(),
        action: "forge_gem".to_string(),
    };

    res = res.add_message(msg_make_randomess);

    RANDOM_JOBS.save(deps.storage, request_forge_hash.clone(), &random_jobs)?;

    DRAND_ROUND_WITH_HASH.save(
        deps.storage,
        request_forge_hash.clone(),
        &drand_round.to_string(),
    )?;

    Ok(res
        .add_attribute("action", "forge_gem")
        .add_attribute("request_forge_hash", request_forge_hash)
        .add_attribute("after", after.seconds().to_string())
        .add_attribute("drand_round", drand_round.to_string()))
}

pub fn execute_select_jackpot_gems(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    campaign_id: String,
) -> Result<Response, ContractError> {
    // Load the config
    let config = CONFIG.load(deps.storage)?;
    // Only contract owner can forge gem
    // ensure_eq!(
    //     info.sender,
    //     config.contract_operator,
    //     ContractError::Unauthorized {}
    // );
    // if campaign_id already exists in RANDOM_JOBS then return error
    if RANDOM_JOBS.may_load(deps.storage, campaign_id.clone())?.is_some() {
        return Err(ContractError::InvalidCampaignId {});
    }

    // Load the nois_proxy
    let nois_proxy = config.nois_proxy;

    let funds = info.funds;

    let mut res = Response::new();

    let after = env.block.time;

    // Make randomness request message to NOIS proxy contract
    let msg_make_randomess = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nois_proxy.into(),
        msg: to_json_binary(&ProxyExecuteMsg::GetRandomnessAfter {
            job_id: campaign_id.clone(),
            after,
        })?,
        funds,
    });

    res = res.add_message(msg_make_randomess);

    let random_jobs = RandomJobs {
        randomness: "waiting...".to_string(),
        action: "get_jackpot_gems".to_string(),
    };

    RANDOM_JOBS.save(deps.storage, campaign_id.clone(), &random_jobs)?;

    Ok(res
        .add_attribute("action", "forge_gem")
        .add_attribute("request_get_jackpot_hash", campaign_id)
        .add_attribute("after", after.seconds().to_string()))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    nois_proxy: Option<String>,
    operator: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    ensure_eq!(
        info.sender,
        config.contract_operator,
        ContractError::Unauthorized {}
    );

    if let Some(nois_proxy) = nois_proxy {
        let nois_proxy = addr_validate(deps.api, &nois_proxy)?;
        config.nois_proxy = nois_proxy;
    }

    if let Some(operator) = operator {
        config.contract_operator = Addr::unchecked(operator);
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("nois_proxy", config.nois_proxy.to_string())
        .add_attribute("contract_operator", config.contract_operator.to_string()))
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

    // Convert the random seed to string
    let randomness_string: String = callback.randomness.to_hex();

    // update random seed
    RANDOM_SEED.save(deps.storage, &randomness_string)?;

    // get random job
    let random_job = RANDOM_JOBS.load(deps.storage, job_id.clone())?;

    match random_job.action.as_str() {
        "forge_gem" => {
            let random_jobs = RandomJobs {
                randomness: randomness_string.clone(),
                action: "forge_gem".to_string(),
            };
            // update random job
            RANDOM_JOBS.save(deps.storage, job_id.clone(), &random_jobs)?;
        }
        "get_jackpot_gems" => {
            let random_jobs = RandomJobs {
                randomness: randomness_string.clone(),
                action: "get_jackpot_gems".to_string(),
            };
            let jackpot_gems = select_jackpot_gems(callback.randomness)?;
            JACKPOT_GEMS_WITH_CAMPAIGN_ID.save(deps.storage, job_id.clone(), &jackpot_gems)?;
            // update random job
            RANDOM_JOBS.save(deps.storage, job_id.clone(), &random_jobs)?;
        }
        _ => {
            return Err(ContractError::InvalidRandomness {});
        }
    }

    Ok(res
        .add_attribute("action", "nois_receive")
        .add_attribute("job_id", job_id))
}

fn select_jackpot_gems(randomness: HexBinary) -> Result<String, ContractError> {
    let mut randomness_arr: [u8; 32] = randomness
        .to_array()
        .map_err(|_| ContractError::InvalidRandomness {})?;
    let mut jackpot_gems: String = String::new();
    let list_color_weight: Vec<(&str, u32)> = vec![("W", 1), ("B", 1), ("G", 1), ("R", 1)];
    let list_number_weight: Vec<(&str, u32)> = vec![
        ("1", 1),
        ("2", 1),
        ("3", 1),
        ("4", 1),
        ("5", 1),
        ("6", 1),
        ("7", 1),
    ];
    for i in 0..3 {
        // define random provider from the random_seed
        let mut provider = sub_randomness_with_key(randomness_arr, i.to_string());
        // random a new randomness
        randomness_arr = provider.provide();
        // randomly selecting an element from list_color_weight
        let color = select_from_weighted(randomness_arr, &list_color_weight).unwrap();
        // randomly selecting an element from list_number_weight
        let number = select_from_weighted(randomness_arr, &list_number_weight).unwrap();
        // append color and number to jackpot_gem for each round
        if i == 2 {
            jackpot_gems = jackpot_gems + &color + &number;
        } else {
            jackpot_gems = jackpot_gems + &color + &number + "-";
        }
    }
    Ok(jackpot_gems)
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
        QueryMsg::GetJackpotGems { campaign_id } => {
            to_json_binary(&query_jackpot_gems(deps, campaign_id)?)
        }
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
) -> StdResult<RandomResponse> {
    let random_job = RANDOM_JOBS.load(deps.storage, request_forge_hash.clone())?;
    let drand_round = DRAND_ROUND_WITH_HASH.load(deps.storage, request_forge_hash.clone())?;
    let random_response = RandomResponse {
        request_forge_hash,
        random_seed: random_job.randomness,
        drand_round,
    };
    Ok(random_response)
}

fn query_jackpot_gems(deps: Deps, campaign_id: String) -> StdResult<String> {
    let jackpot_gems = JACKPOT_GEMS_WITH_CAMPAIGN_ID.load(deps.storage, campaign_id)?;
    Ok(jackpot_gems)
}

/// validate string if it is valid bench32 string addresss
fn addr_validate(api: &dyn Api, addr: &str) -> Result<Addr, ContractError> {
    let addr = api
        .addr_validate(addr)
        .map_err(|_| ContractError::InvalidAddress {})?;
    Ok(addr)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

// Unit test for select_gem_rewards
#[cfg(test)]
mod test_nois_receive {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, HexBinary, Timestamp,
    };
    use nois::NoisCallback;

    use crate::state::{Config, CONFIG, RANDOM_JOBS};

    #[test]
    fn test_update_config() {
        let mut deps = mock_dependencies();

        let env = mock_env();
        let sender = "contract_operator".to_string();
        let funds = vec![];

        let info = mock_info(&sender, &funds);
        let nois_proxy = "nois_proxy".to_string();
        let operator = "contract_operator".to_string();
        let res = super::execute_update_config(deps.as_mut(), info, Some(nois_proxy), Some(operator))
            .unwrap();

        // assert operator and nois_proxy
        // assert_eq!(res.attributes.)
    }

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
                contract_operator: Addr::unchecked("contract_operator"),
            }
        };
        CONFIG.save(&mut deps.storage, &config).unwrap();
        let job_id = "job_id".to_string();
        RANDOM_JOBS
            .save(
                &mut deps.storage,
                job_id.clone(),
                &super::RandomJobs {
                    randomness: "waiting...".to_string(),
                    action: "forge_gem".to_string(),
                },
            )
            .unwrap();

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

    #[test]
    fn test_select_jackpot_gems() {
        let randomness =
            "46FAF1CD4845AB7C5A9DAA7D272259682BF84176A2658DE67CB1317A22134973".to_string();
        let randomness = HexBinary::from_hex(&randomness).unwrap();
        let jackpot_gems = super::select_jackpot_gems(randomness).unwrap();
        assert_eq!(jackpot_gems, "R4-B3-G6");
    }
}
