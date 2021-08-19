use std::collections::HashMap;

use typings::fixed::solarsystem::Solarsystem;
use typings::frontread::site_log::{SiteLog, SiteLogActor};
use typings::persist::player::Player;
use typings::persist::player_location::{PlayerLocation, PlayerLocationSite, PlayerLocationWarp};
use typings::persist::ship::Ship;
use typings::persist::site::Site;
use typings::persist::site_entity::SiteEntity;

use super::entities;

pub fn out(
    solarsystem: Solarsystem,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player_ships: &mut HashMap<Player, Ship>,
    player: Player,
    towards: Site,
    site_log: &mut Vec<SiteLog>,
) {
    let ship = player_ships
        .get(&player)
        .expect("player has to be in player_ships");
    if ship.status.is_alive() {
        entities::remove_player(site_entities, player);
        player_locations.insert(
            player,
            PlayerLocation::Warp(PlayerLocationWarp {
                solarsystem,
                towards,
            }),
        );
        site_log.push(SiteLog::WarpOut(SiteLogActor::Player((
            player,
            ship.fitting.layout,
        ))));
    }
}

/// Add players in warp to the site to the site
pub fn in_site(
    solarsystem: Solarsystem,
    site: Site,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player_ships: &mut HashMap<Player, Ship>,
    players_warping_in: &[Player],
    site_log: &mut Vec<SiteLog>,
) {
    for player in players_warping_in.iter().copied() {
        let ship = player_ships
            .get(&player)
            .expect("player warping in has to be in player_ships");

        site_entities.push(SiteEntity::Player(player));
        player_locations.insert(
            player,
            PlayerLocation::Site(PlayerLocationSite { solarsystem, site }),
        );
        site_log.push(SiteLog::WarpIn(SiteLogActor::Player((
            player,
            ship.fitting.layout,
        ))));
    }
}
