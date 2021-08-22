use space_game_typings::fixed::Statics;
use space_game_typings::persist::player_location::{PlayerLocation, PlayerLocationWarp};
use space_game_typings::persist::site::Site;
use space_game_typings::site::Entity;

use crate::persist::player::write_player_location;
use crate::persist::site::{read_site_entities, read_sites_everywhere};

use super::player::read_all_player_locations;

/// Ensure the `PlayerLocation` exists.
/// Ensure the site the player is in knows its in.
/// Also every strange finding will be printed to stderr.
pub fn ensure_player_locations(statics: &Statics) -> anyhow::Result<()> {
    let all_sites = read_sites_everywhere(&statics.solarsystems);
    let player_locations = read_all_player_locations();

    for (player, location) in player_locations {
        let location_exists = match location {
            PlayerLocation::Warp(warp) => all_sites
                .iter()
                .any(|o| o.0 == warp.solarsystem && o.1 == warp.towards),
            PlayerLocation::Station(_) => {
                // TODO: ensure station still exists
                true
            }
            PlayerLocation::Site(pls) => {
                let site = all_sites
                    .iter()
                    .find(|o| o.0 == pls.solarsystem && o.1 == pls.site);
                if let Some((solarsystem, site)) = site {
                    let entities = read_site_entities(*solarsystem, *site)
                        .expect("site exists so its entities should too");
                    let site_knows = entities
                        .iter()
                        .any(|o| matches!(o, Entity::Player((p, _)) if p == &player));
                    if !site_knows {
                        eprintln!(
                            "    player expected to be in site but site didnt knew: {:?} {} {:?}",
                            player, solarsystem, site
                        );
                        write_player_location(player, PlayerLocation::default())?;
                    }
                    true
                } else {
                    false
                }
            }
        };
        if !location_exists {
            let solarsystem = location.solarsystem();
            eprintln!(
                "    player expected to be in an non existing site. Bring player to existing site. {:?} was here: {:?}",
                player, location
            );
            let first_safe = all_sites
                .iter()
                .find(|o| o.0 == solarsystem && matches!(o.1, Site::Station(_) | Site::Stargate(_)))
                .expect("system shouldve had at least a stargate");
            write_player_location(
                player,
                PlayerLocation::Warp(PlayerLocationWarp {
                    solarsystem,
                    towards: first_safe.1,
                }),
            )?;
        }
    }

    Ok(())
}
