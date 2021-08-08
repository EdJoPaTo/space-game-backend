use std::collections::HashMap;

use typings::fixed::{solarsystem, Statics};
use typings::persist::site;
use typings::persist::site_entity::SiteEntity;

use crate::math::round::advance;
use crate::persist::player::{
    add_player_in_warp, pop_players_in_warp, read_player_instructions, read_player_location,
    read_player_ship, write_player_instructions, write_player_location, write_player_ship,
};
use crate::persist::site::{read_site_entities, read_sites, write_site_entities};

pub fn all(statics: &Statics) -> anyhow::Result<()> {
    let mut some_error = false;
    for solarsystem in statics.solarsystems.keys().copied() {
        let sites = read_sites(solarsystem).expect("init at least created gate sites");
        for site_info in sites.values().flatten() {
            if let Err(err) = handle(statics, solarsystem, site_info) {
                some_error = true;
                eprintln!("ERROR gameloop::site::handle {}", err);
            }
        }
    }
    if some_error {
        Err(anyhow::anyhow!("ERROR gameloop::site::all had some error"))
    } else {
        Ok(())
    }
}

pub fn handle(
    statics: &Statics,
    solarsystem: solarsystem::Identifier,
    site_info: &site::Info,
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
    let players_warping_in = pop_players_in_warp(solarsystem, site_unique);
    let all_players_involved = {
        players_in_site
            .iter()
            .chain(&players_warping_in)
            .collect::<Vec<_>>()
    };

    let mut instructions = {
        let mut result = HashMap::new();
        for player in &players_in_site {
            let instructions = read_player_instructions(player);
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

    let outputs = advance(
        statics,
        solarsystem,
        site_info,
        &mut site_entities,
        &mut instructions,
        &mut player_locations,
        &mut player_ships,
        &players_warping_in,
    )?;

    // Nothing after this point is allowed to fail the rest -> Data has to be saved
    let mut some_error = false;
    let error_prefix = format!("ERROR handle site {} {}", solarsystem, site_unique);

    if let Err(err) = write_site_entities(solarsystem, site_unique, &site_entities) {
        some_error = true;
        eprintln!("{} write_site_entities {}", error_prefix, err);
    }
    for (player, instructions) in &instructions {
        if let Err(err) = write_player_instructions(player, instructions) {
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
    for (solarsystem, site_unique, player) in outputs.warp_out {
        if let Err(err) = add_player_in_warp(solarsystem, &site_unique, player) {
            some_error = true;
            eprintln!("{} add_player_in_warp {}", error_prefix, err);
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
