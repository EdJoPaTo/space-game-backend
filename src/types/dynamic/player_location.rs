use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use super::ship::{ShipFitting, ShipStatus};
use super::site::SiteInfo;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", untagged)]
pub enum PlayerLocation {
    Site(PlayerInSite),
    Space(PlayerInSpace),
    Station(PlayerAtStation),
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

#[cfg(test)]
fn dummy_fitting() -> ShipFitting {
    ShipFitting {
        layout: "shiplayoutRookieShip".to_string(),
        slots_targeted: vec![],
        slots_self: vec![],
        slots_passive: vec![],
    }
}

#[cfg(test)]
fn dummy_status() -> ShipStatus {
    ShipStatus {
        capacitor: 42,
        hitpoints_armor: 42,
        hitpoints_structure: 42,
    }
}

#[test]
fn can_identify_in_site() -> anyhow::Result<()> {
    let data = PlayerLocation::Site(PlayerInSite {
        solarsystem: "bla".to_string(),
        site: SiteInfo {
            kind: crate::types::fixed::site::SiteKind::AsteroidField,
            unique: "666".to_string(),
            name: None,
        },
        ship_fitting: dummy_fitting(),
        ship_status: dummy_status(),
    });
    let json = serde_json::to_string_pretty(&data)?;
    println!("json {}", json);
    let some = serde_json::from_str::<PlayerLocation>(&json)?;
    if let PlayerLocation::Site(_) = some {
        Ok(())
    } else {
        panic!("wrong!");
    }
}

#[test]
fn can_identify_in_space() -> anyhow::Result<()> {
    let data = PlayerLocation::Space(PlayerInSpace {
        solarsystem: "bla".to_string(),
        ship_fitting: dummy_fitting(),
        ship_status: dummy_status(),
    });
    let json = serde_json::to_string_pretty(&data)?;
    println!("json {}", json);
    let some = serde_json::from_str::<PlayerLocation>(&json)?;
    if let PlayerLocation::Space(_) = some {
        Ok(())
    } else {
        panic!("wrong!");
    }
}

#[test]
fn can_identify_station() -> anyhow::Result<()> {
    let data = PlayerLocation::Station(PlayerAtStation {
        solarsystem: "bla".to_string(),
        station: 2,
    });
    let json = serde_json::to_string_pretty(&data)?;
    println!("json {}", json);
    let some = serde_json::from_str::<PlayerLocation>(&json)?;
    if let PlayerLocation::Station(_) = some {
        Ok(())
    } else {
        panic!("wrong!");
    }
}
