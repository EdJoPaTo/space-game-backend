use std::collections::HashMap;

use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::player::location::{
    PlayerLocation, PlayerLocationSite, PlayerLocationStation, PlayerLocationWarp,
};
use space_game_typings::site::instruction::Instruction;
use space_game_typings::site::{advance, Entity, Log, Site};

use crate::persist::player::{read_player_site_instructions, write_player_site_instructions};
use crate::persist::site::{
    add_entity_warping, pop_entity_warping, read_site_entities, read_sites_everywhere, remove_site,
    write_site_entities,
};
use crate::persist::Persist;

mod npc_instructions;

pub fn all(statics: &Statics, persist: &mut Persist) {
    for (solarsystem, site) in read_sites_everywhere(&statics.solarsystems) {
        handle(statics, persist, solarsystem, site).unwrap_or_else(|err| {
            panic!(
                "gameloop::site::handle {:?} {:?} {}",
                solarsystem, site, err
            );
        });
    }
}

#[allow(clippy::too_many_lines)]
fn handle(
    statics: &Statics,
    persist: &mut Persist,
    solarsystem: Solarsystem,
    site: Site,
) -> anyhow::Result<()> {
    let output = {
        let site_entities = read_site_entities(solarsystem, site).unwrap();

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

        let mut output = advance(statics, solarsystem, site, &site_entities, &instructions);

        let mut warping_in = pop_entity_warping(solarsystem, site)?;
        for entity in &warping_in {
            output.log.push(Log::WarpIn(entity.into()));
        }
        output.remaining.append(&mut warping_in);

        output
    };

    if !output.log.is_empty() {
        println!(
            "site_log {:>15} {:?} {} {:?}",
            solarsystem.to_string(),
            site,
            output.log.len(),
            output.log,
        );
    }

    for player in output.dead {
        persist
            .player_notifications
            .add(player, output.log.clone())?;
        write_player_site_instructions(player, &[])?;
        // TODO: home station
        persist
            .player_locations
            .write(player, PlayerLocation::default())?;
    }

    for (solarsystem, station, entity) in output.docking {
        if let Entity::Player((player, ship)) = entity {
            persist
                .player_notifications
                .add(player, output.log.clone())?;
            write_player_site_instructions(player, &[])?;
            persist.player_locations.write(
                player,
                PlayerLocation::Station(PlayerLocationStation {
                    solarsystem,
                    station,
                }),
            )?;

            let mut assets = persist
                .player_station_assets
                .read(player, solarsystem, station);
            assets.ships.push(ship);
            persist
                .player_station_assets
                .write(player, solarsystem, station, &assets)?;
        }
    }

    for (solarsystem, site, entity) in output.warping_out {
        if let Entity::Player((player, _)) = entity {
            persist
                .player_notifications
                .add(player, output.log.clone())?;
            write_player_site_instructions(player, &[])?;
            persist.player_locations.write(
                player,
                PlayerLocation::Warp(PlayerLocationWarp {
                    solarsystem,
                    towards: site,
                }),
            )?;
        }

        add_entity_warping(solarsystem, site, entity)?;
    }

    for entity in &output.remaining {
        if let Entity::Player((player, _)) = entity {
            persist
                .player_notifications
                .add(*player, output.log.clone())?;
            write_player_site_instructions(*player, &[])?;
            persist.player_locations.write(
                *player,
                PlayerLocation::Site(PlayerLocationSite { solarsystem, site }),
            )?;
        }
    }

    if output.remaining.is_empty() {
        remove_site(solarsystem, site)?;
    } else {
        write_site_entities(solarsystem, site, &output.remaining)?;
    }

    Ok(())
}
