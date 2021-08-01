use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Service {
    Dock,
    Jump,
}

#[derive(Debug, Serialize, Deserialize, TS, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum FacilityIdentifier {
    #[serde(rename = "facilityStargate")]
    Stargate,
    #[serde(rename = "facilityStation")]
    Station,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Facility {
    pub services: Vec<Service>,
}

export! {
    Service => "facility-service.ts",
    FacilityIdentifier => "facility-identifier.ts",
    Facility => "facility.ts",
}
