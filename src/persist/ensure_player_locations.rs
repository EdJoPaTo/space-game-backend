use typings::fixed::site::Kind;
use typings::fixed::Statics;
use typings::persist::player_location::PlayerLocation;
use typings::persist::ship::Ship;
use typings::persist::site_entity::{Player, SiteEntity};

use crate::persist::player::{read_player_ship, write_player_location};
use crate::persist::site::{read_site_entities, read_sites_everywhere, write_site_entities};

use super::player::read_all_player_locations;

/// Ensure the `PlayerLocation` exists.
/// Ensure the site the player is in knows its in.
/// Also every strange finding will be printed to stderr.
pub fn ensure_player_locations(statics: &Statics) -> anyhow::Result<()> {
    let all_sites = read_sites_everywhere(&statics.solarsystems);
    let player_locations = read_all_player_locations();

    for (player, location) in &player_locations {
        let location_exists = match location {
            PlayerLocation::Warp(warp) => all_sites
                .iter()
                .any(|o| o.0 == warp.solarsystem && o.1.site_unique == warp.towards_site_unique),
            PlayerLocation::Station(_) => {
                // TODO: ensure station still exists
                true
            }
            PlayerLocation::Site(site) => {
                let site = all_sites
                    .iter()
                    .find(|o| o.0 == site.solarsystem && o.1.site_unique == site.site_unique);
                if let Some((solarsystem, site_info)) = site {
                    let mut entities = read_site_entities(*solarsystem, &site_info.site_unique)
                        .expect("site exists so its entities should too");
                    let site_knows = entities
                        .iter()
                        .any(|o| matches!(o, SiteEntity::Player(p) if &p.id == player));
                    if !site_knows {
                        eprintln!("ensure_player_locations player expected to be in site but site didnt knew: {} {} {:?}", player, solarsystem, site_info);
                        let player_ship =
                            read_player_ship(player).unwrap_or_else(|_| Ship::default(statics));

                        entities.push(SiteEntity::Player(Player {
                            id: player.to_string(),
                            shiplayout: player_ship.fitting.layout,
                        }));
                        write_site_entities(*solarsystem, &site_info.site_unique, &entities)?;
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
                "ensure_player_locations player expected to be in an non existing site. Bring player to existing site. {} was here: {:?}",
                player, location
            );
            let first_safe = all_sites
                .iter()
                .find(|o| o.0 == solarsystem && matches!(o.1.kind, Kind::Station | Kind::Stargate))
                .expect("system shouldve had at least a stargate");
            write_player_location(
                player,
                &PlayerLocation::Warp(typings::persist::player_location::Warp {
                    solarsystem,
                    towards_site_unique: first_safe.1.site_unique.to_string(),
                }),
            )?;
        }
    }

    Ok(())
}
