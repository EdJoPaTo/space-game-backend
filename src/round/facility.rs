use std::collections::HashMap;

use typings::fixed::solarsystem::Solarsystem;
use typings::persist::player::Player;
use typings::persist::player_location::{PlayerLocation, Station, Warp};
use typings::persist::site::{self, Info};
use typings::persist::site_entity::SiteEntity;

use super::entities;

pub fn jump(
    solarsystem: Solarsystem,
    site_info: &site::Info,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player: Player,
) {
    let target_solarsystem = site_info
        .site_unique
        .trim_start_matches("stargate")
        .parse()
        .unwrap_or_else(|_| {
            panic!(
                "stargate site_unique is formatted differently than expected {}",
                site_info.site_unique
            );
        });
    let target_site = Info::generate_stargate(solarsystem);
    entities::remove_player(site_entities, player);
    player_locations.insert(
        player,
        PlayerLocation::Warp(Warp {
            solarsystem: target_solarsystem,
            towards_site_unique: target_site.site_unique,
        }),
    );
}

pub fn dock(
    solarsystem: Solarsystem,
    _site_info: &site::Info,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player: Player,
) {
    entities::remove_player(site_entities, player);
    player_locations.insert(
        player,
        PlayerLocation::Station(Station {
            solarsystem,
            // TODO: dock at correct station
            station: 0,
        }),
    );
}
