use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::frontread::site_entity::{Lifeless, Npc, Player, SiteEntity};
use space_game_typings::persist::site::Site;

use crate::persist::player::read_player_ship;
use crate::persist::site::read_site_entities;

pub fn read(statics: &Statics, solarsystem: Solarsystem, site: Site) -> Vec<SiteEntity> {
    let persist_entities = read_site_entities(solarsystem, site).unwrap_or_default();
    let mut result = Vec::new();
    for entity in &persist_entities {
        result.push(match entity {
            space_game_typings::persist::site_entity::SiteEntity::Facility(info) => {
                SiteEntity::Facility(info.into())
            }
            space_game_typings::persist::site_entity::SiteEntity::Lifeless(info) => {
                SiteEntity::Lifeless(Lifeless::new(&statics.lifeless, info))
            }
            space_game_typings::persist::site_entity::SiteEntity::Npc(info) => {
                SiteEntity::Npc(Npc::new(statics, info))
            }
            space_game_typings::persist::site_entity::SiteEntity::Player(player) => {
                let ship = read_player_ship(*player);
                SiteEntity::Player(Player::new(statics, *player, &ship))
            }
        });
    }
    result
}
