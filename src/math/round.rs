#![allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]

use std::collections::HashMap;

use typings::fixed::facility::Service;
use typings::fixed::solarsystem::Solarsystem;
use typings::fixed::Statics;
use typings::frontrw::site_instruction::SiteInstruction;
use typings::persist::player;
use typings::persist::player_location::{PlayerLocation, Station, Warp};
use typings::persist::ship::{Ship, Status};
use typings::persist::site::{self, Info};
use typings::persist::site_entity::{Npc, Player, SiteEntity};

use super::effect::{apply_to_origin, apply_to_target};

pub struct Outputs {}

pub fn advance(
    statics: &Statics,
    solarsystem: Solarsystem,
    site_info: &site::Info,
    site_entities: &mut Vec<SiteEntity>,
    instructions: &mut HashMap<player::Identifier, Vec<SiteInstruction>>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player_ships: &mut HashMap<player::Identifier, Ship>,
    players_warping_in: &[player::Identifier],
) -> anyhow::Result<Outputs> {
    // TODO: npcs need instructions too…
    // TODO: some instructions are standalone. Warp and nothing else for example. Idea: dont allow warp when some effect is there

    let sorted_instructions = super::site_instructions::sort(instructions);
    if !sorted_instructions.is_empty() {
        println!(
            "site::handle {:>15} {:20} {:?}",
            solarsystem.to_string(),
            site_info.site_unique,
            sorted_instructions
        );
    }

    for (player, instruction) in &sorted_instructions {
        let origin_ship = player_ships
            .get_mut(player)
            .expect("player_ships has to contain player with instructions");
        let location = player_locations
            .get_mut(player)
            .expect("player with instructions has to be in player_locations");

        match instruction {
            SiteInstruction::ModuleUntargeted(module) => {
                if let Some(module) = origin_ship
                    .fitting
                    .slots_untargeted
                    .get(module.module_index as usize)
                    .and_then(|o| statics.modules_untargeted.get(o))
                {
                    if let Some(my_new_status) =
                        apply_to_origin(origin_ship.status, &module.effects)
                    {
                        origin_ship.status = my_new_status;
                    }
                } else {
                    println!("WARN: player untargeted module not handled {:?}", module);
                }
            }
            SiteInstruction::ModuleTargeted(module) => {
                let target_site_index = module.target_index_in_site as usize;
                if let Some(module) = origin_ship
                    .fitting
                    .slots_targeted
                    .get(module.module_index as usize)
                    .and_then(|o| statics.modules_targeted.get(o))
                {
                    if let Some(target) = site_entities.get_mut(target_site_index) {
                        if let Some(origin_new_status) =
                            apply_to_origin(origin_ship.status, &module.effects_origin)
                        {
                            origin_ship.status = origin_new_status;
                            match target {
                                SiteEntity::Facility(_) | SiteEntity::Lifeless(_) => { /* Currently immune */
                                }
                                SiteEntity::Npc(npc) => {
                                    npc.status =
                                        apply_to_target(npc.status, &module.effects_target);
                                }
                                SiteEntity::Player(player) => {
                                    let target_ship = player_ships
                                        .get_mut(&player.id)
                                        .expect("player in site has to be in player_ships");
                                    target_ship.status =
                                        apply_to_target(target_ship.status, &module.effects_target);
                                }
                            }
                        }
                    }
                } else {
                    println!("WARN: player targeted module not handled {:?}", module);
                }
            }
            SiteInstruction::Facility(facility) => {
                // TODO: ensure still alive
                match facility.service {
                    Service::Dock => {
                        remove_player_from_entities(site_entities, player);
                        *location = PlayerLocation::Station(Station {
                            solarsystem,
                            // TODO: dock at correct station
                            station: 0,
                        });
                    }
                    Service::Jump => {
                        let target_solarsystem = site_info
                            .site_unique
                            .trim_start_matches("stargate")
                            .parse()
                            .unwrap_or_else(|_| panic!("stargate site_unique is formatted differently than expected {}", site_info.site_unique));
                        let target_site = Info::generate_stargate(solarsystem);

                        remove_player_from_entities(site_entities, player);
                        *location = PlayerLocation::Warp(Warp {
                            solarsystem: target_solarsystem,
                            towards_site_unique: target_site.site_unique,
                        });
                    }
                }
            }
            SiteInstruction::Warp(warp) => {
                assert_ne!(
                    warp.site_unique, site_info.site_unique,
                    "players warping in are not yet in instructions"
                );
                // TODO: ensure still alive
                remove_player_from_entities(site_entities, player);
                *location = PlayerLocation::Warp(Warp {
                    solarsystem,
                    towards_site_unique: warp.site_unique.to_string(),
                });
            }
        }
    }

    // provide ship/passive quality boni
    for ship in player_ships.values_mut() {
        if let Some(layout) = statics.ship_layouts.get(&ship.fitting.layout) {
            if let Some(my_new_status) = apply_to_origin(ship.status, &layout.round_effects) {
                ship.status = my_new_status;
            }
        }
    }

    *site_entities = cleanup_entities(statics, site_entities, player_ships)?;

    // Add players in warp to here
    for player in players_warping_in {
        site_entities.push(SiteEntity::Player(Player {
            id: player.to_string(),
        }));
        player_locations.insert(
            player.to_string(),
            PlayerLocation::Site(site::Identifier {
                solarsystem,
                site_unique: site_info.site_unique.to_string(),
            }),
        );
    }

    // Clear instructions
    // TODO: keep something like warp
    for (_player, instructions) in instructions.iter_mut() {
        instructions.clear();
    }

    Ok(Outputs {})
}

fn player_pos(site_entities: &[SiteEntity], player: &str) -> Option<usize> {
    site_entities
        .iter()
        .position(|o| matches!(o, SiteEntity::Player(p) if p.id == player))
}

fn remove_player_from_entities(site_entities: &mut Vec<SiteEntity>, player: &str) {
    if let Some(index) = player_pos(site_entities, player) {
        site_entities.remove(index);
    }
}

#[allow(clippy::option_if_let_else)]
/// cleanup dead and ensure status is within ship layout limits
fn cleanup_entities(
    statics: &Statics,
    before: &[SiteEntity],
    player_ships: &mut HashMap<player::Identifier, Ship>,
) -> anyhow::Result<Vec<SiteEntity>> {
    let mut remaining = Vec::new();
    for entity in before {
        match entity {
            SiteEntity::Facility(_) => {
                remaining.push(entity.clone());
            }
            SiteEntity::Lifeless(l) => {
                if l.status.is_alive() {
                    remaining.push(entity.clone());
                }
            }
            SiteEntity::Npc(npc) => {
                if let Some(status) = npc.status.min_layout(statics, &npc.fitting) {
                    if status.is_alive() {
                        remaining.push(SiteEntity::Npc(Npc {
                            faction: npc.faction,
                            fitting: npc.fitting.clone(),
                            status,
                        }));
                    }
                }
            }
            SiteEntity::Player(p) => {
                let ship = player_ships
                    .get_mut(&p.id)
                    .expect("player has to be in player_ships");
                ship.status = if let Some(status) = ship.status.min_layout(statics, &ship.fitting) {
                    if status.is_alive() {
                        remaining.push(entity.clone());
                    }
                    status
                } else {
                    Status {
                        capacitor: 0,
                        hitpoints_armor: 0,
                        hitpoints_structure: 0,
                    }
                };

                // TODO: when dead: location isnt site anymore
                // TODO: when dead: ship is default now
            }
        }
    }
    Ok(remaining)
}
