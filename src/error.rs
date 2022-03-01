use cosmwasm_std::{Addr, StdError};
use cw_controllers::{AdminError, HookError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Hook(#[from] HookError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Host-opponent pair already has a game going on")]
    HostOpponentPairAlreadyHasGame {},

    #[error("The host address has been blacklisted")]
    HostAddressBlacklisted {},

    #[error("No game found for the host opponent pair")]
    NoGameFoundForHostOpponentPair {
        host_address: Addr,
        opponent_address: Addr,
    },

    #[error("Opponent played an invalid move")]
    InvalidMove {
        msg: String
    }
}
