use std::collections::HashMap;

use typings::fixed::solarsystem::Solarsystem;
use typings::fixed::Statics;
use typings::persist::player::Player;
use typings::persist::player_location::PlayerLocation;
use typings::persist::site::Site;
use typings::persist::site_entity::SiteEntity;

use crate::persist::player::{
    add_player_site_log, read_all_player_locations, read_player_location, read_player_ship,
    read_player_site_instructions, write_player_location, write_player_ship,
    write_player_site_instructions,
};
use crate::persist::site::{
    read_site_entities, read_sites_everywhere, remove_site, write_site_entities,
};
use crate::round::advance;

mod npc_instructions;

pub fn all(statics: &Statics) -> anyhow::Result<()> {
    let mut some_error = false;

    let mut players_in_warp = Vec::new();
    for (player, location) in read_all_player_locations() {
        let solarsystem = location.solarsystem();
        match location {
            PlayerLocation::Site(_) | PlayerLocation::Station(_) => {}
            PlayerLocation::Warp(warp) => {
                players_in_warp.push((player, solarsystem, warp.towards));
            }
        }
    }

    for (solarsystem, site) in read_sites_everywhere(&statics.solarsystems) {
        let players_warping_in = players_in_warp
            .iter()
            .filter(|o| o.1 == solarsystem && o.2 == site)
            .map(|o| o.0)
            .collect::<Vec<_>>();
        if let Err(err) = handle(statics, solarsystem, site, &players_warping_in) {
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

#[allow(clippy::too_many_lines)]
fn handle(
    statics: &Statics,
    solarsystem: Solarsystem,
    site: Site,
    players_warping_in: &[Player],
) -> anyhow::Result<()> {
    let mut site_entities = read_site_entities(solarsystem, site).unwrap();

    let players_in_site = {
        let mut result = Vec::new();
        for entity in &site_entities {
            if let SiteEntity::Player(p) = entity {
                result.push(*p);
            }
        }
        result
    };
    let all_players_involved = {
        players_in_site
            .iter()
            .chain(players_warping_in)
            .copied()
            .collect::<Vec<_>>()
    };

    let mut player_instructions = {
        let mut result = HashMap::new();
        for player in players_in_site {
            let instructions = read_player_site_instructions(player);
            result.insert(player, instructions);
        }
        result
    };

    let npc_instructions = npc_instructions::generate(site, &site_entities);

    let mut player_ships = HashMap::new();
    let mut player_locations = HashMap::new();

    for player in all_players_involved.iter().copied() {
        let ship = read_player_ship(player);
        player_ships.insert(player, ship);
        let location = read_player_location(player);
        player_locations.insert(player, location);
    }

    let outputs = advance(
        statics,
        solarsystem,
        site,
        &mut site_entities,
        &mut player_instructions,
        &npc_instructions,
        &mut player_locations,
        &mut player_ships,
        players_warping_in,
    );

    if !outputs.site_log.is_empty() {
        println!(
            "site_log {:>15} {:?} {} {:?}",
            solarsystem.to_string(),
            site,
            outputs.site_log.len(),
            outputs.site_log,
        );
    }

    // Nothing after this point is allowed to fail the rest -> Data has to be saved
    let mut some_error = false;
    let error_prefix = format!("ERROR handle site {} {:?}", solarsystem, site);

    if site_entities.is_empty() {
        println!(
            "gameloop::site_round Remove empty site {} {:?}",
            solarsystem, site
        );
        if let Err(err) = remove_site(solarsystem, site) {
            some_error = true;
            eprintln!("{} remove_site {}", error_prefix, err);
        }
    } else if let Err(err) = write_site_entities(solarsystem, site, &site_entities) {
        some_error = true;
        eprintln!("{} write_site_entities {}", error_prefix, err);
    }

    for (player, instructions) in player_instructions {
        if let Err(err) = write_player_site_instructions(player, &instructions) {
            some_error = true;
            eprintln!(
                "{} write_player_instructions {:?} {}",
                error_prefix, player, err
            );
        }
    }
    for (player, ship) in player_ships {
        if let Err(err) = write_player_ship(player, &ship) {
            some_error = true;
            eprintln!("{} write_player_ship {:?} {}", error_prefix, player, err);
        }
    }
    for (player, location) in player_locations {
        if let Err(err) = write_player_location(player, location) {
            some_error = true;
            eprintln!(
                "{} write_player_location {:?} {}",
                error_prefix, player, err
            );
        }
    }
    for player in all_players_involved {
        if let Err(err) = add_player_site_log(player, &outputs.site_log) {
            some_error = true;
            eprintln!("{} add_player_sitelog {:?} {}", error_prefix, player, err);
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