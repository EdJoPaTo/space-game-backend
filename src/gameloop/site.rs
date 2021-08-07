use std::collections::HashMap;
use std::time::Instant;

use typings::fixed::Statics;
use typings::persist::site;
use typings::persist::site_entity::SiteEntity;

use crate::math::round::advance;
use crate::persist::player::{
    add_player_in_warp, pop_players_in_warp, read_player_instructions, read_player_location,
    read_player_ship, write_player_instructions, write_player_location, write_player_ship,
};
use crate::persist::site::{read_site_entities, write_site_entities};

pub fn handle(statics: &Statics, site_identifier: &site::Identifier) -> anyhow::Result<()> {
    println!("handle site {:?}", site_identifier);

    let solarsystem = site_identifier.solarsystem;
    let site_unique = &site_identifier.site_unique;

    let mut measure = Instant::now();

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

    let measure_load = measure.elapsed();
    measure = Instant::now();

    let outputs = advance(
        statics,
        site_identifier,
        &mut site_entities,
        &mut instructions,
        &mut player_locations,
        &mut player_ships,
        &players_warping_in,
    )?;

    let measure_math = measure.elapsed();
    measure = Instant::now();

    write_site_entities(solarsystem, site_unique, &site_entities)?;
    for (player, instructions) in &instructions {
        write_player_instructions(player, instructions)?;
    }
    for (player, ship) in &player_ships {
        write_player_ship(player, ship)?;
    }
    for (player, location) in &player_locations {
        write_player_location(player, location)?;
    }
    for (solarsystem, site_unique, player) in outputs.warp_out {
        add_player_in_warp(solarsystem, &site_unique, player)?;
    }

    let measure_save = measure.elapsed();

    println!(
        "handle site {:?} took {:?} {:?} {:?}",
        site_identifier, measure_load, measure_math, measure_save
    );

    Ok(())
}
