use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ModulePassive {
    pub required_cpu: u32,
    pub required_powergrid: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacitor: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hitpoints_armor: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSelf {
    pub required_cpu: u32,
    pub required_powergrid: u32,

    pub energy_consumption: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub armor_repair: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ModuleTargeted {
    pub required_cpu: u32,
    pub required_powergrid: u32,

    pub energy_consumption: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_mined: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<u32>,
}

export! {
    ModulePassive => "module-passive.ts",
    ModuleSelf => "module-self.ts",
    ModuleTargeted => "module-targeted.ts",
}
