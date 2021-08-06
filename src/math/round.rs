#![allow(dead_code, unused_variables, clippy::unnecessary_wraps, unused_imports)]

use std::collections::HashMap;

use typings::fixed::facility::Service;
use typings::fixed::site::Kind;
use typings::fixed::{solarsystem, Statics};
use typings::frontrw::instruction::{Instruction, ModuleTargeted, ModuleUntargeted};
use typings::persist::player;
use typings::persist::player_location::{PlayerLocation, Station, Warp};
use typings::persist::ship::{Fitting, Ship, Status};
use typings::persist::site::{self, Info};
use typings::persist::site_entity::{Player, SiteEntity};

// TODO: has to be argument or return value
use crate::persist::player::add_player_in_warp;

pub fn advance(
    statics: &Statics,
    site_identifier: &site::Identifier,
    site_entities: &mut Vec<SiteEntity>,
    instructions: &HashMap<player::Identifier, Vec<Instruction>>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player_ships: &mut HashMap<player::Identifier, Ship>,
    players_warping_in: &[player::Identifier],
) -> anyhow::Result<()> {
    // TODO: npcs need instructions too…
    // TODO: some instructions are standalone. Warp and nothing else for example. Idea: mark a player "interactive" and abort warp when something affected the player?

    for (player, instruction) in sorted_instructions(instructions) {
        let location = player_locations
            .entry(player.to_string())
            .or_insert_with(|| {
                PlayerLocation::Station(Station {
                    solarsystem: site_identifier.solarsystem,
                    station: 0,
                })
            });
        let station = if let PlayerLocation::Station(station) = location {
            station.station
        } else {
            0
        };

        match instruction {
            Instruction::ModuleUntargeted(module) => {
                if let Some(module) = player_ships
                    .get(&player)
                    .and_then(|o| o.fitting.slots_untargeted.get(module.module_index as usize))
                    .and_then(|o| statics.modules_untargeted.get(o))
                {
                    // TODO: module energy consumption or something
                }
            }
            Instruction::ModuleTargeted(_) => {
                // TODO
            }
            Instruction::Undock => {
                *location = PlayerLocation::Site(site::Identifier {
                    solarsystem: site_identifier.solarsystem,
                    site_unique: Info::generate_station(site_identifier.solarsystem, station)
                        .site_unique,
                });
            }
            Instruction::Facility(facility) => {
                match facility.service {
                    Service::Dock => {
                        remove_player_from_entities(site_entities, &player);
                        *location = PlayerLocation::Station(Station {
                            solarsystem: site_identifier.solarsystem,
                            station,
                        });
                    }
                    Service::Jump => {
                        let target_solarsystem = site_identifier
                        .site_unique
                        .trim_start_matches("stargate")
                        .parse()
                        .unwrap_or_else(|_| panic!("stargate site_unique is formatted differently than expected {}", site_identifier.site_unique));
                        let target_site = Info::generate_stargate(site_identifier.solarsystem);

                        remove_player_from_entities(site_entities, &player);
                        *location = PlayerLocation::Warp(Warp {
                            solarsystem: target_solarsystem,
                        });
                        add_player_in_warp(target_solarsystem, &target_site.site_unique, player)?;
                    }
                }
            }
            Instruction::Warp(warp) => {
                remove_player_from_entities(site_entities, &player);
                *location = PlayerLocation::Warp(Warp {
                    solarsystem: site_identifier.solarsystem,
                });
                add_player_in_warp(
                    site_identifier.solarsystem,
                    &site_identifier.site_unique,
                    player,
                )?;
            }
        }
    }

    // Add players in warp to here
    for player in players_warping_in {
        site_entities.push(SiteEntity::Player(Player {
            id: player.to_string(),
            shiplayout: player_ships
                .get(player)
                .expect("player warping in also has to be in player_ships")
                .fitting
                .layout
                .to_string(),
        }));
        player_locations.insert(
            player.to_string(),
            PlayerLocation::Site(site::Identifier {
                solarsystem: site_identifier.solarsystem,
                site_unique: site_identifier.site_unique.to_string(),
            }),
        );
    }

    Ok(())
}

fn remove_player_from_entities(site_entities: &mut Vec<SiteEntity>, player: &str) {
    if let Some(index) = site_entities
        .iter()
        .position(|o| matches!(o, SiteEntity::Player(p) if p.id == player))
    {
        site_entities.remove(index);
    }
}

fn sorted_instructions(
    instructions: &HashMap<player::Identifier, Vec<Instruction>>,
) -> Vec<(player::Identifier, Instruction)> {
    let mut result: Vec<(player::Identifier, Instruction)> = Vec::new();
    for (player, instructions) in instructions {
        for instruction in instructions {
            result.push((player.to_string(), instruction.clone()));
        }
    }
    result.sort_by(|a, b| a.1.cmp(&b.1));
    result
}

#[test]
fn sorted_works() {
    let mut example = HashMap::new();
    example.insert(
        "player1".to_string(),
        vec![
            Instruction::Undock,
            Instruction::ModuleUntargeted(ModuleUntargeted { module_index: 0 }),
        ],
    );
    example.insert(
        "player2".to_string(),
        vec![Instruction::ModuleTargeted(ModuleTargeted {
            module_index: 0,
            target_index_in_site: 0,
        })],
    );
    let sorted = sorted_instructions(&example);
    assert_eq!(sorted.len(), 3);
    assert_eq!(
        sorted[0],
        (
            "player1".to_string(),
            Instruction::ModuleUntargeted(ModuleUntargeted { module_index: 0 })
        )
    );
    assert_eq!(
        sorted[1],
        (
            "player2".to_string(),
            Instruction::ModuleTargeted(ModuleTargeted {
                module_index: 0,
                target_index_in_site: 0,
            })
        )
    );
    assert_eq!(sorted[2], ("player1".to_string(), Instruction::Undock));
}
