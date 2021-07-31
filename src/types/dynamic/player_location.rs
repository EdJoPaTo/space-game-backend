use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use super::ship::{ShipFitting, ShipStatus};
use super::site::SiteInfo;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum PlayerLocation {
    Station(PlayerAtStation),
    Space(PlayerInSpace),
    Site(PlayerInSite),
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlayerAtStation {
    pub solarsystem: String,
    pub station: u8,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInSpace {
    pub solarsystem: String,

    pub ship_fitting: ShipFitting,
    pub ship_status: ShipStatus,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInSite {
    pub solarsystem: String,
    pub site: SiteInfo,

    pub ship_fitting: ShipFitting,
    pub ship_status: ShipStatus,
}

export! {
    PlayerLocation => "player-location.ts",
    PlayerAtStation => "player-at-station.ts",
    PlayerInSpace => "player-in-space.ts",
    PlayerInSite => "player-in-site.ts",
}
