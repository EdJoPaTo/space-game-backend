use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use crate::types::fixed::facility::FacilityIdentifier;
use crate::types::fixed::site::SiteKind;

// TODO: the frontend dont care for internals like fittings.
// Split up typings and have one internal and one external?
// For now… just share everything witht the frontend.

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SiteInfo {
    pub kind: SiteKind,
    pub unique: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteInternals {
    pub entities: Vec<SiteEntity>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SiteEntity {
    Facility(SiteEntityFacility),
    Lifeless(SiteEntityLifeless),
    Npc(SiteEntityNpc),
    Player(SiteEntityPlayer),
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SiteEntityFacility {
    pub id: FacilityIdentifier,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SiteEntityLifeless {
    pub id: String,
    // TODO: status like hitpoints?
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SiteEntityNpc {
    pub shiplayout: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SiteEntityPlayer {
    pub id: String,
    pub shiplayout: String,
}

export! {
    SiteInfo => "site-info.ts",
    SiteEntityFacility => "site-entity-facility.ts",
    SiteEntityLifeless => "site-entity-lifeless.ts",
    SiteEntityNpc => "site-entity-npc.ts",
    SiteEntityPlayer => "site-entity-player.ts",
}

#[test]
fn can_parse() -> anyhow::Result<()> {
    let origin = SiteEntity::Lifeless(SiteEntityLifeless {
        id: "lifelessAsteroid".to_string(),
    });
    let json = serde_json::to_string_pretty(&origin)?;
    println!("json {}", json);

    let some = serde_json::from_str::<SiteEntity>(&json)?;
    println!("some {:?}", some);

    if let SiteEntity::Lifeless(v) = some {
        assert_eq!(v.id, "lifelessAsteroid");
        Ok(())
    } else {
        panic!();
    }
}
