use typings::fixed::{solarsystem, Statics};
use typings::frontread::site_entity::{Lifeless, Npc, Player, SiteEntity};

use crate::persist::player::read_player_ship;
use crate::persist::site::read_site_entities;

pub fn read(
    statics: &Statics,
    solarsystem: solarsystem::Identifier,
    site_unique: &str,
) -> Vec<SiteEntity> {
    let persist_entities = read_site_entities(solarsystem, site_unique).unwrap_or_default();
    let mut result = Vec::new();
    for entity in &persist_entities {
        result.push(match entity {
            typings::persist::site_entity::SiteEntity::Facility(info) => {
                SiteEntity::Facility(info.into())
            }
            typings::persist::site_entity::SiteEntity::Lifeless(info) => {
                SiteEntity::Lifeless(Lifeless::new(&statics.lifeless, info))
            }
            typings::persist::site_entity::SiteEntity::Npc(info) => {
                SiteEntity::Npc(Npc::new(statics, info))
            }
            typings::persist::site_entity::SiteEntity::Player(info) => {
                let ship = read_player_ship(&info.id);
                SiteEntity::Player(Player::new(statics, info, &ship))
            }
        });
    }
    result
}
