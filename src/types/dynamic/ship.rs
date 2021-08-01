use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ShipFitting {
    pub layout: String,

    pub slots_targeted: Vec<String>,
    pub slots_self: Vec<String>,
    pub slots_passive: Vec<String>,
}

/// The current situation of the ship.
/// For the totals check the `ShipFitting`.
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ShipStatus {
    pub capacitor: u32,
    pub hitpoints_armor: u32,
    pub hitpoints_structure: u32,
}

export! {
    ShipFitting => "ship-fitting.ts",
    ShipStatus => "ship-status.ts",
}
