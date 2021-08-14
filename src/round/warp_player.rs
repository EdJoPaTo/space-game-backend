use std::collections::HashMap;

use typings::fixed::solarsystem::Solarsystem;
use typings::persist::player::Player;
use typings::persist::player_location::{PlayerLocation, Warp};
use typings::persist::site;
use typings::persist::site_entity::SiteEntity;

use super::entities;

pub fn out(
    solarsystem: Solarsystem,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player: Player,
    target_site_unique: &str,
) {
    entities::remove_player(site_entities, player);
    player_locations.insert(
        player,
        PlayerLocation::Warp(Warp {
            solarsystem,
            towards_site_unique: target_site_unique.to_string(),
        }),
    );
}

/// Add players in warp to the site to the site
pub fn in_site(
    solarsystem: Solarsystem,
    site_info: &site::Info,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    players_warping_in: &[Player],
) {
    for player in players_warping_in.iter().copied() {
        site_entities.push(SiteEntity::Player(player));
        player_locations.insert(
            player,
            PlayerLocation::Site(site::Identifier {
                solarsystem,
                site_unique: site_info.site_unique.to_string(),
            }),
        );
    }
}
