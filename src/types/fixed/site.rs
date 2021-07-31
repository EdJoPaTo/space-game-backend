use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use super::facility::FacilityIdentifier;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum SiteKind {
    Facility(FacilityIdentifier),
    AsteroidField,
}

export! {SiteKind => "site-kind.ts"}
