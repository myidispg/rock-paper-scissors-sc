#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{GameMove, GameState, State, GAMES, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // This is just initializing the contract. No game is started yet.
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame {
            opponent,
            host_move,
        } => try_start_game(deps, info, opponent, host_move),
    }
}

pub fn try_start_game(
    deps: DepsMut,
    info: MessageInfo,
    opponent: Addr,
    host_move: GameMove,
) -> Result<Response, ContractError> {
    // validate opponent address
    let opponent_address = deps.api.addr_validate(&opponent.to_string())?;

    // Make sure that the host-opponent has only one game going on.
    let start_game = |game_state: Option<GameState>| -> Result<GameState, ContractError> {
        match game_state {
            Some(_) => {
                // Error. Host already has a game going on.
                return Err(ContractError::HostOpponentPairAlreadyHasGame {});
            }
            None => {
                // Start game
                let game_state = GameState {
                    host_address: info.sender.clone(),
                    opponent_address: opponent_address.clone(),
                    host_move: Some(host_move),
                    opponent_move: None,
                    result: None,
                };
                return Ok(game_state);
            }
        }
    };

    // This will look for games with the given host-opponent pair.
    GAMES.update(
        deps.storage,
        (info.sender.clone(), opponent_address.clone()),
        start_game,
    )?;

    // Game started successfully.
    return Ok(Response::new()
        .add_attribute("method", "start_game")
        .add_attribute("host", info.sender));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetGame {
            host_address,
            opponent_address,
        } => to_binary(&query_game(deps, host_address, opponent_address)?),
        QueryMsg::GetGameByHost {
            host_address: address,
        } => to_binary(&query_game_by_address(deps, true, address)?),
        QueryMsg::GetGameByOpponent {
            opponent_address: address,
        } => to_binary(&query_game_by_address(deps, false, address)?),
    }
}

pub fn query_game(deps: Deps, host_address: Addr, opponent_address: Addr) -> StdResult<GameState> {
    let game_state = GAMES
        .may_load(deps.storage, (host_address, opponent_address))?
        .unwrap();
    let game_response = GameState {
        host_address: game_state.host_address,
        opponent_address: game_state.opponent_address,
        host_move: game_state.host_move,
        opponent_move: game_state.opponent_move,
        result: game_state.result,
    };
    return Ok(game_response);
}

pub fn query_game_by_address(deps: Deps, host: bool, address: Addr) -> StdResult<Vec<GameState>> {
    // if "host" is true, match by host address, else by opponent address.

    // Make sure the address is valid
    let address = deps.api.addr_validate(&address.to_string())?;

    let mut game_states: Vec<GameState> = Vec::new();

    // Search by host address
    if host == true {
        // Prefix allows to return only those games that have the "address" as the first value in the key tuple.
        let game_state_keys: StdResult<Vec<_>> = GAMES
            .prefix(address.clone())
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect();
        
        for game_state_key in &game_state_keys? {
            game_states.push(game_state_key.1.clone());
        }
    } else {
        // Search by borrower address
        let game_state_keys: StdResult<Vec<_>> = GAMES.range(deps.storage, None, None, cosmwasm_std::Order::Ascending).collect();
        println!("Game state keys by opponent: {:?}", game_state_keys);

        for game_state_key in &game_state_keys? {
            if game_state_key.1.opponent_address == address {
                game_states.push(game_state_key.1.clone());
            }
        }

    }
    return Ok(game_states);
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn test_game_start() {
        let mut deps = mock_dependencies(&[]);

        // Instantiate the contract
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uluna"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg);

        // Start a game
        let host_info = mock_info("host", &coins(0, "uluna"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("opponent"),
            host_move: GameMove::Rock,
        };

        let _res = execute(deps.as_mut(), mock_env(), host_info, msg);

        // Try starting another game
        // let host_info = mock_info("host", &coins(0, "uluna"));
        // let msg = ExecuteMsg::StartGame {
        //     opponent: "opponent".to_string(),
        //     host_move: GameMove::Rock,
        // };
        // let _res = execute(deps.as_mut(), mock_env(), host_info, msg);
        // match _res {
        //     Err(ContractError::HostAlreadyHasGame{}) => {
        //         println!("Second execute result: {:?}", ContractError::HostAlreadyHasGame{});
        //         assert!(false);
        //     }
        //     Ok(response) => {
        //         println!("Second execute result: {:?}", response);
        //     }
        //     _ => {
        //         println!("Second execute has something unforeseen");
        //     }
        // }
    }
    #[test]
    fn test_game_query_pair() {
        // Query for a game using a specific pair.
        let mut deps = mock_dependencies(&[]);

        // Instantiate the contract
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uluna"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg);

        // Start a game
        let host_info = mock_info("host", &coins(0, "uluna"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("opponent"),
            host_move: GameMove::Rock,
        };

        let _res = execute(deps.as_mut(), mock_env(), host_info, msg);

        // Query for the game
        let msg = QueryMsg::GetGame {
            host_address: Addr::unchecked("host"),
            opponent_address: Addr::unchecked("opponent"),
        };

        let _res = query(deps.as_ref(), mock_env(), msg).unwrap();
    }

    #[test]
    fn test_game_query_address() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uluna"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg);

        // Start a game
        let host_info = mock_info("host", &coins(0, "uluna"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("opponent"),
            host_move: GameMove::Rock,
        };
        let _res = execute(deps.as_mut(), mock_env(), host_info, msg);
        // Start another game. This should not work
        let host_info = mock_info("creator", &coins(0, "uluna"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("opponent2"),
            host_move: GameMove::Rock,
        };
        let _res = execute(deps.as_mut(), mock_env(), host_info, msg);
        match _res {
            Err(ContractError::HostOpponentPairAlreadyHasGame {}) => {
                panic!("The host opponent pair already has a game going on!");
            }

            _ => {
                // println!("Second execute result: {:?}", _res);
            }
        }

        // Query for the games
        // let msg = QueryMsg::GetGameByOpponent {
        //     opponent_address: Addr::unchecked("opponent3"),
        // };
        let msg = QueryMsg::GetGameByHost {
            host_address: Addr::unchecked("creator"),
        };
        let _res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let _res: Vec<GameState> = from_binary(&_res).unwrap();
        println!("Games found: {:?}", _res);
    }

    //     #[test]
    //     fn test_valid_address() {
    //         let mut deps = mock_dependencies(&[]);

    //         let msg = InstantiateMsg {};
    //         let info = mock_info("creator", &coins(0, "token"));
    //         let _res = instantiate(deps.as_mut(), mock_env(), info, msg);

    //         // check if opponent address is valid
    //         let opponent_info = mock_info("opponent", &coins(0, "token"));
    //         let msg = ExecuteMsg::StartGame {
    //             opponent: String::from("what-users-provide"),
    //         };

    //         let _res = execute(deps.as_mut(), mock_env(), opponent_info, msg);

    //         println!("{:?}", _res);
    //     }
}
