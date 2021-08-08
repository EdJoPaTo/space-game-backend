use std::collections::HashMap;

use typings::fixed::{solarsystem, Statics};
use typings::persist::player_location::PlayerLocation;
use typings::persist::site_entity::SiteEntity;
use typings::persist::{player, site};

use crate::math::round::advance;
use crate::persist::player::{
    read_all_player_locations, read_player_location, read_player_ship,
    read_player_site_instructions, write_player_location, write_player_ship,
    write_player_site_instructions,
};
use crate::persist::site::{read_site_entities, read_sites_everywhere, write_site_entities};

pub fn all(statics: &Statics) -> anyhow::Result<()> {
    let mut some_error = false;

    let mut players_in_warp = Vec::new();
    for (player, location) in read_all_player_locations() {
        let solarsystem = location.solarsystem();
        match location {
            PlayerLocation::Site(_) | PlayerLocation::Station(_) => {}
            PlayerLocation::Warp(warp) => {
                players_in_warp.push((player, solarsystem, warp.towards_site_unique));
            }
        }
    }

    for (solarsystem, site_info) in read_sites_everywhere(&statics.solarsystems) {
        let players_warping_in = players_in_warp
            .iter()
            .filter(|o| o.1 == solarsystem && o.2 == site_info.site_unique)
            .map(|o| o.0.to_string())
            .collect::<Vec<_>>();
        if let Err(err) = handle(statics, solarsystem, &site_info, &players_warping_in) {
            some_error = true;
            eprintln!("ERROR gameloop::site::handle {}", err);
        }
    }
    if some_error {
        Err(anyhow::anyhow!("ERROR gameloop::site::all had some error"))
    } else {
        Ok(())
    }
}

fn handle(
    statics: &Statics,
    solarsystem: solarsystem::Identifier,
    site_info: &site::Info,
    players_warping_in: &[player::Identifier],
) -> anyhow::Result<()> {
    let site_unique = &site_info.site_unique;

    let mut site_entities = read_site_entities(solarsystem, site_unique).unwrap_or_default();

    let players_in_site = {
        let mut result = Vec::new();
        for entity in &site_entities {
            if let SiteEntity::Player(p) = entity {
                result.push(p.id.to_string());
            }
        }
        result
    };
    let all_players_involved = {
        players_in_site
            .iter()
            .chain(players_warping_in)
            .collect::<Vec<_>>()
    };

    let mut instructions = {
        let mut result = HashMap::new();
        for player in &players_in_site {
            let instructions = read_player_site_instructions(player);
            result.insert(player.to_string(), instructions);
        }
        result
    };

    let mut player_ships = HashMap::new();
    let mut player_locations = HashMap::new();

    for player in all_players_involved {
        let ship = read_player_ship(player)?;
        player_ships.insert(player.to_string(), ship);
        let location = read_player_location(player)?;
        player_locations.insert(player.to_string(), location);
    }

    let _outputs = advance(
        statics,
        solarsystem,
        site_info,
        &mut site_entities,
        &mut instructions,
        &mut player_locations,
        &mut player_ships,
        players_warping_in,
    )?;

    // Nothing after this point is allowed to fail the rest -> Data has to be saved
    let mut some_error = false;
    let error_prefix = format!("ERROR handle site {} {}", solarsystem, site_unique);

    if let Err(err) = write_site_entities(solarsystem, site_unique, &site_entities) {
        some_error = true;
        eprintln!("{} write_site_entities {}", error_prefix, err);
    }
    for (player, instructions) in &instructions {
        if let Err(err) = write_player_site_instructions(player, instructions) {
            some_error = true;
            eprintln!(
                "{} write_player_instructions {} {}",
                error_prefix, player, err
            );
        }
    }
    for (player, ship) in &player_ships {
        if let Err(err) = write_player_ship(player, ship) {
            some_error = true;
            eprintln!("{} write_player_ship {} {}", error_prefix, player, err);
        }
    }
    for (player, location) in &player_locations {
        if let Err(err) = write_player_location(player, location) {
            some_error = true;
            eprintln!("{} write_player_location {} {}", error_prefix, player, err);
        }
    }

    if some_error {
        Err(anyhow::anyhow!(
            "{} some error while saving occured",
            error_prefix
        ))
    } else {
        Ok(())
    }
}
