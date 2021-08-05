#![allow(dead_code, unused_variables, clippy::unnecessary_wraps, unused_imports)]

use std::collections::HashMap;

use typings::fixed::site::Kind;
use typings::fixed::{solarsystem, Statics};
use typings::frontrw::instruction::{
    Instruction, ModuleTargeted, ModuleUntargeted, Movement, Targeted, Untargeted,
};
use typings::persist::player;
use typings::persist::player_location::{PlayerLocation, Site, Station, Warp};
use typings::persist::ship::{Fitting, Ship, Status};
use typings::persist::site::Info;
use typings::persist::site_entity::{Player, SiteEntity};

pub fn advance(
    statics: &Statics,
    solarsystem: solarsystem::Identifier,
    site_unique: &str,
    site_entities: &mut Vec<SiteEntity>,
    instructions: &HashMap<player::Identifier, Vec<Instruction>>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player_ships: &mut HashMap<player::Identifier, Ship>,
) {
    // TODO: npcs need instructions too…
    // TODO: some instructions are standalone. Warp and nothing else for example. Idea: mark a player "interactive" and abort warp when something affected the player?

    for (player, instruction) in sorted_instructions(instructions) {
        match instruction {
            Instruction::Untargeted(i) => match i {
                Untargeted::Module(module) => {
                    if let Some(module) = player_ships
                        .get(&player)
                        .and_then(|o| o.fitting.slots_untargeted.get(module.module_index as usize))
                        .and_then(|o| statics.modules_untargeted.get(o))
                    {
                        // TODO: module energy consumption or something
                    }
                }
            },
            Instruction::Targeted(i) => todo!(),
            Instruction::Movement(i) => {
                let location = player_locations
                    .entry(player.to_string())
                    .or_insert_with(|| {
                        PlayerLocation::Station(Station {
                            solarsystem,
                            station: 0,
                        })
                    });

                match i {
                    Movement::Undock => {
                        let station = if let PlayerLocation::Station(station) = location {
                            station.station
                        } else {
                            0
                        };
                        *location = PlayerLocation::Site(Site {
                            solarsystem,
                            site_unique: Info::generate_station(solarsystem, station).unique,
                        });
                    }
                    Movement::Warp(warp) => {
                        if warp.site_unique == site_unique {
                            // The target is this site -> Add to site
                            site_entities.push(SiteEntity::Player(Player {
                                id: player.to_string(),
                                shiplayout: player_ships
                                    .get(&player)
                                    .unwrap()
                                    .fitting
                                    .layout
                                    .to_string(),
                            }));
                            *location = PlayerLocation::Site(Site {
                                solarsystem,
                                site_unique: site_unique.to_string(),
                            });
                        } else {
                            // Target is another site -> Enter warp
                            if let Some(index) = site_entities
                                .iter()
                                .position(|o| matches!(o, SiteEntity::Player(p) if p.id == player))
                            {
                                site_entities.remove(index);
                            }
                            *location = PlayerLocation::Warp(Warp {
                                solarsystem,
                                towards_site_unique: warp.site_unique.to_string(),
                            });
                        }
                    }
                }
            }
        }
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
            Instruction::Movement(Movement::Undock),
            Instruction::Untargeted(Untargeted::Module(ModuleUntargeted { module_index: 0 })),
        ],
    );
    example.insert(
        "player2".to_string(),
        vec![Instruction::Targeted(Targeted::Module(ModuleTargeted {
            module_index: 0,
            target_index_in_site: 0,
        }))],
    );
    let sorted = sorted_instructions(&example);
    assert_eq!(sorted.len(), 3);
    assert_eq!(
        sorted[0],
        (
            "player1".to_string(),
            Instruction::Untargeted(Untargeted::Module(ModuleUntargeted { module_index: 0 }))
        )
    );
    assert_eq!(
        sorted[1],
        (
            "player2".to_string(),
            Instruction::Targeted(Targeted::Module(ModuleTargeted {
                module_index: 0,
                target_index_in_site: 0,
            }))
        )
    );
    assert_eq!(
        sorted[2],
        (
            "player1".to_string(),
            Instruction::Movement(Movement::Undock)
        )
    );
}
