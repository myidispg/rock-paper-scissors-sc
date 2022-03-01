#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw0::maybe_addr;
use cw2::set_contract_version;
use cw_controllers::AdminResponse;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{GameMove, GameResult, GameState, State, ADMIN, GAMES, HOOKS, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // This is just initializing the contract. No game is started yet.
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin_address = maybe_addr(deps.api, Some(info.sender.to_string()))?;
    ADMIN.set(deps.branch(), admin_address)?;
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
    let api = deps.api;

    match msg {
        ExecuteMsg::StartGame {
            opponent,
            host_move,
        } => try_start_game(deps, info, opponent, host_move),
        ExecuteMsg::UpdateAdmin { admin } => try_update_admin(deps, info, admin),
        ExecuteMsg::AddHook { hook_address } => {
            Ok(HOOKS.execute_add_hook(&ADMIN, deps, info, api.addr_validate(&hook_address)?)?)
        }
        ExecuteMsg::RemoveHook { hook_address } => {
            Ok(HOOKS.execute_remove_hook(&ADMIN, deps, info, api.addr_validate(&hook_address)?)?)
        }
        ExecuteMsg::OpponentMove {
            host_address,
            opponent_address,
            opponent_move,
        } => try_opponent_move(deps, info, host_address, opponent_address, opponent_move),
    }
}

pub fn try_opponent_move(
    deps: DepsMut,
    info: MessageInfo,
    host_address: String,
    opponent_address: String,
    opponent_move: GameMove,
) -> Result<Response, ContractError> {
    /*
    Steps:
    1. Check if opponent move is valid
    2. Check if the opponent and host addresses are valid
    3. Check if there is a game between the host and the opponent
    4. Compare the host and the opponent move
    5. Update the game state to reflect the winner.
    6. No need to return the state as the game can be queried later to see who won.
    */

    if opponent_move != GameMove::Paper
        && opponent_move != GameMove::Rock
        && opponent_move != GameMove::Scissors
    {
        return Err(ContractError::InvalidMove {
            msg: "Opponent played an invalid move".to_string(),
        });
    }

    let host_address = deps.api.addr_validate(&host_address.to_string())?;
    let opponent_address = deps.api.addr_validate(&opponent_address.to_string())?;

    let game_state = query_game(
        deps.as_ref(),
        host_address.clone(),
        opponent_address.clone(),
    );

    match game_state {
        Err(_) => {
            // This will be there if there is no game found for the pair
            return Err(ContractError::NoGameFoundForHostOpponentPair {
                host_address: host_address.clone(),
                opponent_address: opponent_address.clone(),
            });
        }
        // Proceed ahead if there is a game found
        _ => {}
    }

    println!("Found game: {:?}", game_state);

    // Compare the opponent moves
    let found_game: GameState = update_opponent_move(game_state.unwrap(), opponent_move.clone());
    // Update the game state in storage
    // Make sure that the host-opponent has only one game going on.
    let update_state = |game_state: Option<GameState>| -> Result<GameState, ContractError> {
        match game_state {
            Some(_) => {
                // Modify game
                let game_state = GameState {
                    host_address: host_address.clone(),
                    opponent_address: opponent_address.clone(),
                    host_move: found_game.host_move,
                    opponent_move: Some(opponent_move),
                    result: found_game.result,
                };
                return Ok(game_state);
            }
            None => {
                // Error. Host already has a game going on.
                return Err(ContractError::HostOpponentPairAlreadyHasGame {});
            }
        }
    };
    GAMES.update(
        deps.storage,
        (host_address.clone(), opponent_address.clone()),
        update_state,
    )?;

    // Game started successfully.
    return Ok(Response::new()
        .add_attribute("method", "opponent_move")
        .add_attribute("host", info.sender)
        .add_attribute("opponent", opponent_address));
}

fn update_opponent_move(mut game_state: GameState, opponent_move: GameMove) -> GameState {
    // Check the winning conditions for each move and mark it as such. Mark draw as required
    if opponent_move == GameMove::Rock {
        if game_state.host_move == Some(GameMove::Scissors) {
            game_state.result = Some(GameResult::OpponentWins);
        } else if game_state.host_move == Some(GameMove::Rock) {
            game_state.result = Some(GameResult::Tie);
        } else {
            game_state.result = Some(GameResult::HostWins);
        }
    } else if opponent_move == GameMove::Paper {
        if game_state.host_move == Some(GameMove::Rock) {
            game_state.result = Some(GameResult::OpponentWins);
        } else if game_state.host_move == Some(GameMove::Paper) {
            game_state.result = Some(GameResult::Tie);
        } else {
            game_state.result = Some(GameResult::HostWins);
        }
    } else if opponent_move == GameMove::Scissors {
        if game_state.host_move == Some(GameMove::Paper) {
            game_state.result = Some(GameResult::OpponentWins);
        } else if game_state.host_move == Some(GameMove::Scissors) {
            game_state.result = Some(GameResult::Tie);
        } else {
            game_state.result = Some(GameResult::HostWins);
        }
    }

    return game_state;
}

pub fn try_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    admin: Addr,
) -> Result<Response, ContractError> {
    let admin_address = maybe_addr(deps.api, Some(admin.to_string()))?;

    return Ok(ADMIN.execute_update_admin(deps, info, admin_address)?);
}

pub fn try_start_game(
    deps: DepsMut,
    info: MessageInfo,
    opponent: Addr,
    host_move: GameMove,
) -> Result<Response, ContractError> {
    // Check if the host is blacklisted
    let hooks = HOOKS.query_hooks(deps.as_ref())?.hooks;
    for blacklist_address in hooks.iter() {
        if blacklist_address == &info.sender {
            // No game can be started by a blacklisted address
            return Err(ContractError::HostAddressBlacklisted {});
        }
    }

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
        QueryMsg::GetAdmin {} => to_binary(&query_admin(deps)?),
    }
}

pub fn query_admin(deps: Deps) -> StdResult<Addr> {
    let admin_address = ADMIN.query_admin(deps.clone())?.admin.unwrap();
    let admin_address = maybe_addr(deps.api, Some(admin_address))?.unwrap();
    return Ok(admin_address);
}

pub fn query_game(deps: Deps, host_address: Addr, opponent_address: Addr) -> StdResult<GameState> {
    let game_state = GAMES
        .may_load(deps.storage, (host_address, opponent_address))
        .unwrap();

    match game_state {
        Some(game_state) => {
            return Ok(GameState {
                host_address: game_state.host_address,
                opponent_address: game_state.opponent_address,
                host_move: game_state.host_move,
                opponent_move: game_state.opponent_move,
                result: game_state.result,
            });
        }
        None => StdResult::Err(StdError::generic_err("Game not found")),
    }
}

pub fn query_game_by_address(deps: Deps, host: bool, address: Addr) -> StdResult<Vec<GameState>> {
    // if "host" is true, match by host address, else by opponent address.
    // The function could be modified to use the login for searching by opponent address only.
    // But I wanted to let the other method stay here as well.

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
        let game_state_keys: StdResult<Vec<_>> = GAMES
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect();
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
        let mut deps = mock_dependencies();

        // Instantiate the contract
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uluna"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg);

        // Try to add a hook
        let blacklist_info = mock_info("creator", &coins(0, "uluna"));
        let msg = ExecuteMsg::AddHook {
            hook_address: "blacklist".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), blacklist_info, msg);
        match _res {
            Err(e) => {
                println!("Hook Error: {:?}", e);
            }
            Ok(_) => {
                println!("Hook added successfully");
            }
        }

        // Start a game
        let host_info = mock_info("host", &coins(0, "uluna"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("opponent"),
            host_move: GameMove::Rock,
        };

        let _res = execute(deps.as_mut(), mock_env(), host_info, msg);

        match _res {
            Err(e) => {
                println!("Game Error: {:?}", e);
            }
            Ok(_) => {
                println!("Game started successfully");
            }
        }
    }
    #[test]
    fn test_game_query_pair() {
        // Query for a game using a specific pair.
        let mut deps = mock_dependencies();

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
    fn test_admin() {
        let mut deps = mock_dependencies();

        // Instantiate the contract
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uluna"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg);

        // Query for the admin
        // let host_info = mock_info("host", &coins(0, "uluna"));
        let msg = QueryMsg::GetAdmin {};
        let _res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let _res: Addr = from_binary(&_res).unwrap();

        // Change admin
        let host_info = mock_info("creator", &coins(0, "uluna"));
        let msg = ExecuteMsg::UpdateAdmin {
            admin: Addr::unchecked("new_admin"),
        };
        let _res = execute(deps.as_mut(), mock_env(), host_info, msg);

        match _res {
            Err(e) => {
                println!("Error while updating admin: {:?}", e);
            }
            _ => println!("Successfully updated the admin"),
        }
    }

    #[test]
    fn test_opponent_move() {
        let mut deps = mock_dependencies();
        // Instantiating the contract
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
        // Make an opponent move
        let host_info = mock_info("opponent", &coins(0, "uluna"));
        let msg = ExecuteMsg::OpponentMove {
            host_address: String::from("host"),
            opponent_address: String::from("opponent"),
            opponent_move: GameMove::Scissors,
        };
        let _res = execute(deps.as_mut(), mock_env(), host_info, msg);
        match _res {
            Err(e) => {
                println!("Error while making opponent move: {:?}", e);
            }
            _ => println!("Successfully made opponent move"),
        }

        // Query the game state to see who won that game.
        let host_info = mock_info("host", &coins(0, "uluna"));
        let msg = QueryMsg::GetGame {
            host_address: Addr::unchecked("host"),
            opponent_address: Addr::unchecked("opponent"),
        };
        let _res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let _res: GameState = from_binary(&_res).unwrap();
        println!("Checking game result: {:?}", _res);
    }

    #[test]
    fn test_game_query_address() {
        let mut deps = mock_dependencies();

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
    }
}
