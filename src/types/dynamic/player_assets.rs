use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use super::ship::ShipFitting;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStationAssets {
    pub solarsystem: String,
    pub station: u8,

    pub ships: Vec<ShipFitting>,
}

export! {PlayerStationAssets => "player-station-assets.ts"}
