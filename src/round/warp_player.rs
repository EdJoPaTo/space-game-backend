use std::collections::HashMap;

use typings::fixed::solarsystem::Solarsystem;
use typings::persist::player::Player;
use typings::persist::player_location::{PlayerLocation, PlayerLocationSite, PlayerLocationWarp};
use typings::persist::site::Site;
use typings::persist::site_entity::SiteEntity;

use super::entities;

pub fn out(
    solarsystem: Solarsystem,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player: Player,
    towards: Site,
) {
    entities::remove_player(site_entities, player);
    player_locations.insert(
        player,
        PlayerLocation::Warp(PlayerLocationWarp {
            solarsystem,
            towards,
        }),
    );
}

/// Add players in warp to the site to the site
pub fn in_site(
    solarsystem: Solarsystem,
    site: Site,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    players_warping_in: &[Player],
) {
    for player in players_warping_in.iter().copied() {
        site_entities.push(SiteEntity::Player(player));
        player_locations.insert(
            player,
            PlayerLocation::Site(PlayerLocationSite { solarsystem, site }),
        );
    }
}
