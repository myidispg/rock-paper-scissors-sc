use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
// Each map has a key: (host_address, opponent_address) -> game_state
pub const GAMES: Map<(Addr, Addr), GameState> = Map::new("games");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameState {
    // When the game starts, host and opponent addresses will definitely be there. 
    // Host move is Option just for uniformity.
    pub host_address: Addr,
    pub opponent_address: Addr,
    pub host_move: Option<GameMove>,
    pub opponent_move: Option<GameMove>,
    pub result: Option<GameResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameMove {
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
    HostWins,
    OpponentWins,
    Tie,
}
