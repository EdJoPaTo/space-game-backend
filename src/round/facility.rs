use std::collections::HashMap;

use typings::fixed::solarsystem::Solarsystem;
use typings::frontread::site_log::{SiteLog, SiteLogActor};
use typings::persist::player::Player;
use typings::persist::player_location::{
    PlayerLocation, PlayerLocationStation, PlayerLocationWarp,
};
use typings::persist::ship::Ship;
use typings::persist::site::Site;
use typings::persist::site_entity::SiteEntity;

use super::entities;

pub fn jump(
    origin_solarsystem: Solarsystem,
    origin_site: Site,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player_ships: &mut HashMap<Player, Ship>,
    player: Player,
    site_log: &mut Vec<SiteLog>,
) {
    if let Site::Stargate(target_solarsystem) = origin_site {
        let ship = player_ships
            .get(&player)
            .expect("player has to be in player_ships");
        if ship.status.is_alive() {
            entities::remove_player(site_entities, player);
            player_locations.insert(
                player,
                PlayerLocation::Warp(PlayerLocationWarp {
                    solarsystem: target_solarsystem,
                    towards: Site::Stargate(origin_solarsystem),
                }),
            );
            site_log.push(SiteLog::Jump(SiteLogActor::Player((
                player,
                ship.fitting.layout,
            ))));
        }
    } else {
        panic!("tried to jump from a site without stargate");
    }
}

pub fn dock(
    solarsystem: Solarsystem,
    _site: Site,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player_ships: &mut HashMap<Player, Ship>,
    player: Player,
    site_log: &mut Vec<SiteLog>,
) {
    let ship = player_ships
        .get(&player)
        .expect("player has to be in player_ships");
    if ship.status.is_alive() {
        entities::remove_player(site_entities, player);
        player_locations.insert(
            player,
            PlayerLocation::Station(PlayerLocationStation {
                solarsystem,
                // TODO: dock at correct station
                station: 0,
            }),
        );
        site_log.push(SiteLog::Dock(SiteLogActor::Player((
            player,
            ship.fitting.layout,
        ))));
    }
}
