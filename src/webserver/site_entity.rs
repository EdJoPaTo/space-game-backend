use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::site::entity_frontread::SiteEntity;
use space_game_typings::site::Site;

use crate::persist::Persist;

pub fn read(
    statics: &Statics,
    persist: &Persist,
    solarsystem: Solarsystem,
    site: Site,
) -> Vec<SiteEntity> {
    persist
        .sites
        .read_entities(solarsystem, site)
        .unwrap_or_default()
        .iter()
        .map(|o| SiteEntity::from(statics, o))
        .collect()
}
