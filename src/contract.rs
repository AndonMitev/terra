use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage,
};

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};
use hex::decode;
use sha2::{Digest, Sha256};

use std::time::{SystemTime, UNIX_EPOCH};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    // TODO: Do validations

    let state = State {
        buyer: deps.api.canonical_address(&msg.buyer)?,
        seller: deps.api.canonical_address(&msg.seller)?,
        expiration: msg.expiration,
        value: msg.value,
        secret_hash: msg.secret_hash,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Claim { secret } => try_claim(deps, env, secret),
        HandleMsg::Refund {} => try_refund(deps, env),
    }
}

pub fn try_claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    secret: String,
) -> StdResult<HandleResponse> {
    let state = config_read(&deps.storage).load()?;

    let mut hasher = Sha256::default();
    let message: Vec<u8> = decode(secret).expect("Invalid Hex String");

    hasher.update(&message);

    let secret_hash: String = format!("{:x}", hasher.finalize());

    if state.secret_hash != secret_hash {
        return Err(StdError::generic_err("Invalid secret"));
    }

    // TODO: Transfer locked value to buyer
    // where this value gonna be locked !?
    // AND VALIDATIONS ON input secret

    Ok(HandleResponse::default())
}

pub fn try_refund<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let state = config_read(&deps.storage).load()?;

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if current_timestamp < state.expiration {
        return Err(StdError::generic_err("Swap is not expired"));
    }

    // TODO: Transfer locked value to buyer
    // where this value gonna be locked !?

    Ok(HandleResponse::default())
}

// pub fn try_increment<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     _env: Env,
// ) -> StdResult<HandleResponse> {
//     // config(&mut deps.storage).update(|mut state| {
//     //     state.count += 1;
//     //     Ok(state)
//     // })?;

//     Ok(HandleResponse::default())
// }

// pub fn try_reset<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     env: Env,
//     count: i32,
// ) -> StdResult<HandleResponse> {
//     // let api = &deps.api;
//     // config(&mut deps.storage).update(|mut state| {
//     //     if api.canonical_address(&env.message.sender)? != state.owner {
//     //         return Err(StdError::unauthorized());
//     //     }
//     //     state.count = count;
//     //     Ok(state)
//     // })?;
//     Ok(HandleResponse::default())
// }

// pub fn query<S: Storage, A: Api, Q: Querier>(
//     deps: &Extern<S, A, Q>,
//     msg: QueryMsg,
// ) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
//     }
// }

// fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
//     // let state = config_read(&deps.storage).load()?;
//     Ok(CountResponse { count: 1 })
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env};
//     use cosmwasm_std::{coins, from_binary, StdError};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(20, &[]);

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // beneficiary can release it
//         let env = mock_env("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Increment {};
//         let _res = handle(&mut deps, env, msg).unwrap();

//         // should increase counter by 1
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // beneficiary can release it
//         let unauth_env = mock_env("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let res = handle(&mut deps, unauth_env, msg);
//         match res {
//             Err(StdError::Unauthorized { .. }) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_env = mock_env("creator", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let _res = handle(&mut deps, auth_env, msg).unwrap();

//         // should now be 5
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
