use std::collections::HashMap;

use typings::fixed::solarsystem::Solarsystem;
use typings::persist::player_location::{PlayerLocation, Warp};
use typings::persist::site_entity::{Player, SiteEntity};
use typings::persist::{player, site};

use super::entities;

pub fn out(
    solarsystem: Solarsystem,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player: &str,
    target_site_unique: &str,
) {
    entities::remove_player(site_entities, player);
    player_locations.insert(
        player.to_string(),
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
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    players_warping_in: &[player::Identifier],
) {
    for player in players_warping_in {
        site_entities.push(SiteEntity::Player(Player {
            id: player.to_string(),
        }));
        player_locations.insert(
            player.to_string(),
            PlayerLocation::Site(site::Identifier {
                solarsystem,
                site_unique: site_info.site_unique.to_string(),
            }),
        );
    }
}
