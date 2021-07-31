use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlayerGeneral {
    pub home_solarsystem: String,
    pub home_station: u8,

    /// Paperclips are the currency
    pub paperclips: u64,
}

export! {PlayerGeneral => "player-general.ts"}
