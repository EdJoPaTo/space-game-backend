use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use crate::types::serde_helper::ordered_map;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Solarsystem {
    pub name: String,
    /// Percentage
    pub security: u8,
    /// Amount
    pub planets: u8,

    /// Gates in the system.
    /// Key: Target System
    /// Value: The planet they are
    #[serde(serialize_with = "ordered_map")]
    pub stargates: HashMap<String, u8>,

    /// Stations and at which planet they are.
    /// Example: [1,3] -> Station 1 is at Planet 1, Station 2 is at Planet 3
    pub stations: Vec<u8>,
}

export! {Solarsystem => "solarsystem.ts"}
