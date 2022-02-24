use crate::state::{GameMove, GameState};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // While starting the game, the host does not need to specify an opponent.
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    StartGame {
        opponent: Addr,
        host_move: GameMove,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGame {
        host_address: Addr,
        opponent_address: Addr,
    },
    GetGameByHost {
        host_address: Addr,
    },
    GetGameByOpponent {
        opponent_address: Addr,
    },
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct GameResponse {
//     pub host_address: String,
//     pub opponent_address: String,
//     pub host_move: Option<GameMove>,
//     pub opponent_move: Option<GameMove>,
//     pub result: Option<GameState>,
// }
