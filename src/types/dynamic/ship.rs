use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use crate::types::fixed::modules::{ModulePassive, ModuleSelf, ModuleTargeted};
use crate::types::fixed::shiplayout::ShipLayout;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ShipFitting {
    pub layout: ShipLayout,

    pub slots_targeted: Vec<ModuleTargeted>,
    pub slots_self: Vec<ModuleSelf>,
    pub slots_passive: Vec<ModulePassive>,
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
