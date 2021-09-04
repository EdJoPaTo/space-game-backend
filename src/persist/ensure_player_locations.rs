use space_game_typings::fixed::Statics;
use space_game_typings::player::location::{PlayerLocation, PlayerLocationWarp};
use space_game_typings::site::{Entity, Site};

use super::Persist;

/// Ensure the `PlayerLocation` exists.
/// Ensure the site the player is in knows its in.
/// Also every strange finding will be printed to stderr.
pub fn ensure_player_locations(statics: &Statics, persist: &mut Persist) -> anyhow::Result<()> {
    let all_sites = persist.sites.read_sites_everywhere(&statics.solarsystems);
    let player_locations = persist.player_locations.read_all();

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
                    let entities = persist
                        .sites
                        .read_entities(*solarsystem, *site)
                        .expect("site exists so its entities should too");
                    let site_knows = entities
                        .iter()
                        .any(|o| matches!(o, Entity::Player((p, _)) if p == &player));
                    if !site_knows {
                        eprintln!(
                            "    player expected to be in site but site didnt knew: {:?} {} {:?}",
                            player, solarsystem, site
                        );
                        persist
                            .player_locations
                            .write(player, PlayerLocation::default())?;
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
            persist.player_locations.write(
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
