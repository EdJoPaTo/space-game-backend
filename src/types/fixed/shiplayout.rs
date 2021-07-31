use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ShipLayout {
    pub slots_targeted: u8,
    pub slots_self: u8,
    pub slots_passive: u8,

    pub cpu: u32,
    pub powergrid: u32,
    pub capacitor: u32,
    pub capacitor_recharge: u32,

    pub hitpoints_armor: u32,
    pub hitpoints_structure: u32,
}

export! {ShipLayout => "ship-layout.ts"}
