use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Lifeless {
    pub hitpoints_armor: u32,
    pub hitpoints_structure: u32,
    // TODO: mineable resources
    // TODO: lootable resources
    // TODO: hackable resources
}

export! {Lifeless => "lifeless.ts"}
