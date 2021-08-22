use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::frontread::site_entity::SiteEntity;
use space_game_typings::persist::site::Site;

use crate::persist::site::read_site_entities;

pub fn read(statics: &Statics, solarsystem: Solarsystem, site: Site) -> Vec<SiteEntity> {
    read_site_entities(solarsystem, site)
        .unwrap_or_default()
        .iter()
        .map(|o| SiteEntity::from(statics, o))
        .collect()
}
