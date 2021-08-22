use std::collections::HashMap;

use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::player::location::{PlayerLocation, PlayerLocationStation};
use space_game_typings::player::Player;
use space_game_typings::site::instruction::Instruction;
use space_game_typings::site::{advance, Entity, Log, Site};

use crate::persist::player::{
    add_player_site_log, read_player_site_instructions, read_station_assets, write_player_location,
    write_player_site_instructions, write_station_assets,
};
use crate::persist::site::{
    add_entity_warping, pop_entity_warping, read_site_entities, read_sites_everywhere, remove_site,
    write_site_entities,
};
mod npc_instructions;

pub fn all(statics: &Statics) -> anyhow::Result<()> {
    let mut some_error = false;

    for (solarsystem, site) in read_sites_everywhere(&statics.solarsystems) {
        if let Err(err) = handle(statics, solarsystem, site) {
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
fn handle(statics: &Statics, solarsystem: Solarsystem, site: Site) -> anyhow::Result<()> {
    let outputs = {
        let mut site_entities = read_site_entities(solarsystem, site).unwrap();

        let mut warping_in = pop_entity_warping(solarsystem, site)?;
        site_entities.append(&mut warping_in);

        let mut instructions: HashMap<usize, Vec<Instruction>> = HashMap::new();

        for (index, entity) in site_entities.iter().enumerate() {
            if let Entity::Player((player, _)) = entity {
                let mut additionals = read_player_site_instructions(*player);
                let all = instructions.entry(index).or_default();
                all.append(&mut additionals);
            }
        }

        for (index, mut additionals) in npc_instructions::generate(site, &site_entities) {
            let all = instructions.entry(index).or_default();
            all.append(&mut additionals);
        }

        advance(statics, solarsystem, site, &site_entities, &instructions)
    };

    if !outputs.log.is_empty() {
        println!(
            "site_log {:>15} {:?} {} {:?}",
            solarsystem.to_string(),
            site,
            outputs.log.len(),
            outputs.log,
        );
    }

    // Nothing after this point is allowed to fail the rest -> Data has to be saved
    let mut some_error = false;
    let error_prefix = format!("ERROR handle site {} {:?}", solarsystem, site);

    for player in outputs.dead {
        handle_player_log_and_instructions(&outputs.log, player, &error_prefix, &mut some_error);
        // TODO: home station
        if let Err(err) = write_player_location(player, PlayerLocation::default()) {
            some_error = true;
            eprintln!("{} docking write_player_location {}", error_prefix, err);
        }
    }

    for (solarsystem, station, entity) in outputs.docking {
        if let Entity::Player((player, ship)) = entity {
            handle_player_log_and_instructions(
                &outputs.log,
                player,
                &error_prefix,
                &mut some_error,
            );
            if let Err(err) = write_player_location(
                player,
                PlayerLocation::Station(PlayerLocationStation {
                    solarsystem,
                    station,
                }),
            ) {
                some_error = true;
                eprintln!("{} docking write_player_location {}", error_prefix, err);
            }

            let mut assets = read_station_assets(player, solarsystem, station);
            assets.ships.push(ship);
            if let Err(err) = write_station_assets(player, solarsystem, station, &assets) {
                some_error = true;
                eprintln!("{} docking write_station_assets {}", error_prefix, err);
            }
        }
    }

    for (solarsystem, site, entity) in outputs.warping_out {
        if let Entity::Player((player, _)) = entity {
            handle_player_log_and_instructions(
                &outputs.log,
                player,
                &error_prefix,
                &mut some_error,
            );
        }

        if let Err(err) = add_entity_warping(solarsystem, site, entity) {
            some_error = true;
            eprintln!("{} add_player_warping {}", error_prefix, err);
        }
    }

    for entity in &outputs.remaining {
        if let Entity::Player((player, _)) = entity {
            handle_player_log_and_instructions(
                &outputs.log,
                *player,
                &error_prefix,
                &mut some_error,
            );
        }
    }

    if outputs.remaining.is_empty() {
        println!(
            "gameloop::site_round Remove empty site {} {:?}",
            solarsystem, site
        );
        if let Err(err) = remove_site(solarsystem, site) {
            some_error = true;
            eprintln!("{} remove_site {}", error_prefix, err);
        }
    } else if let Err(err) = write_site_entities(solarsystem, site, &outputs.remaining) {
        some_error = true;
        eprintln!("{} write_site_entities {}", error_prefix, err);
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

fn handle_player_log_and_instructions(
    log: &[Log],
    player: Player,
    error_prefix: &str,
    some_error: &mut bool,
) {
    if let Err(err) = add_player_site_log(player, log) {
        *some_error = true;
        eprintln!("{} add_player_site_log {:?} {}", error_prefix, player, err);
    }
    if let Err(err) = write_player_site_instructions(player, &[]) {
        *some_error = true;
        eprintln!(
            "{} write_player_instructions {:?} {}",
            error_prefix, player, err
        );
    }
}
