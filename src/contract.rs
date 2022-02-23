#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{GameMove, GameResult, GameState, State, GAMES, STATE};

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
    opponent: String,
    host_move: GameMove,
) -> Result<Response, ContractError> {
    // validate opponent address
    let opponent_address = deps.api.addr_validate(&opponent)?;
    println!("opponent_address: {:?}", opponent_address);

    // Make sure that the host has only one game going on. A host can not have two games at the same time.
    let start_game = |game_state: Option<GameState>| -> Result<GameState, ContractError> {
        match game_state {
            Some(_) => {
                // Error. Host already has a game going on.
                return Err(ContractError::HostAlreadyHasGame {});
            }
            None => {
                // Start game
                let game_state = GameState {
                    host_address: info.sender.clone(),
                    opponent_address: opponent_address,
                    host_move: Some(host_move),
                    opponent_move: None,
                    result: None,
                };
                return Ok(game_state);
            }
        }
    };

    GAMES.update(deps.storage, info.sender.to_string(), start_game)?;

    // Game started successfully.
    return Ok(Response::new()
        .add_attribute("method", "start_game")
        .add_attribute("host", info.sender));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
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
            opponent: "opponent".to_string(),
            host_move: GameMove::Rock,
        };

        let _res = execute(deps.as_mut(), mock_env(), host_info, msg);
        println!("First execute result: {:?}", _res);

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
