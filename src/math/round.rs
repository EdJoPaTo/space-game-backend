#![allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]

use std::collections::HashMap;

use typings::fixed::facility::Service;
use typings::fixed::module::Effect;
use typings::fixed::{solarsystem, Statics};
use typings::frontrw::instruction::Instruction;
use typings::persist::player;
use typings::persist::player_location::{PlayerLocation, Station, Warp};
use typings::persist::ship::{Ship, Status};
use typings::persist::site::{self, Info};
use typings::persist::site_entity::{Npc, Player, SiteEntity};

use crate::math::effect::apply_to_status;

pub struct Outputs {}

pub fn advance(
    statics: &Statics,
    solarsystem: solarsystem::Identifier,
    site_info: &site::Info,
    site_entities: &mut Vec<SiteEntity>,
    instructions: &mut HashMap<player::Identifier, Vec<Instruction>>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player_ships: &mut HashMap<player::Identifier, Ship>,
    players_warping_in: &[player::Identifier],
) -> anyhow::Result<Outputs> {
    // TODO: npcs need instructions too…
    // TODO: some instructions are standalone. Warp and nothing else for example. Idea: dont allow warp when some effect is there

    let sorted_instructions = super::instructions::sort(instructions);
    if !sorted_instructions.is_empty() {
        println!(
            "site::handle {:>15} {:20} {:?}",
            solarsystem.to_string(),
            site_info.site_unique,
            sorted_instructions
        );
    }

    // First collect all module effects
    let mut effects: HashMap<usize, Vec<Effect>> = HashMap::new();
    for (player, instruction) in &sorted_instructions {
        match instruction {
            Instruction::ModuleUntargeted(module) => {
                let origin_site_index =
                    player_pos(site_entities, player).expect("player has to be in site_entities");
                if let Some(module) = player_ships
                    .get(player)
                    .and_then(|o| o.fitting.slots_untargeted.get(module.module_index as usize))
                    .and_then(|o| statics.modules_untargeted.get(o))
                {
                    for effect in &module.effects {
                        effects.entry(origin_site_index).or_default().push(*effect);
                    }
                } else {
                    println!("WARN: player untargeted module not handled {:?}", module);
                }
            }
            Instruction::ModuleTargeted(module) => {
                let origin_site_index =
                    player_pos(site_entities, player).expect("player has to be in site_entities");
                let target_site_index = module.target_index_in_site as usize;
                if let Some(module) = player_ships
                    .get(player)
                    .and_then(|o| o.fitting.slots_targeted.get(module.module_index as usize))
                    .and_then(|o| statics.modules_targeted.get(o))
                {
                    for effect in &module.effects_origin {
                        effects.entry(origin_site_index).or_default().push(*effect);
                    }
                    for effect in &module.effects_target {
                        effects.entry(target_site_index).or_default().push(*effect);
                    }
                } else {
                    println!("WARN: player targeted module not handled {:?}", module);
                }
            }
            Instruction::Facility(_) | Instruction::Undock | Instruction::Warp(_) => {
                // Handled later
            }
        }
    }

    // Then apply the module effects
    for (index, mut effects) in effects {
        effects.sort();
        if let Some(entity) = site_entities.get_mut(index) {
            match entity {
                SiteEntity::Facility(_) | SiteEntity::Lifeless(_) => { /* Currently immune */ }
                SiteEntity::Npc(npc) => {
                    npc.status = apply_to_status(npc.status, &effects);
                }
                SiteEntity::Player(player) => {
                    let ship = player_ships
                        .get_mut(&player.id)
                        .expect("player has to be in player_ships");
                    ship.status = apply_to_status(ship.status, &effects);
                }
            }
        }
    }

    // cleanup dead or impaired
    *site_entities = cleanup_entities(statics, site_entities, player_ships)?;

    // Finally do movements
    for (player, instruction) in &sorted_instructions {
        // TODO: check if player is still in site_entites. If not the player is dead
        let location = player_locations
            .entry(player.to_string())
            .or_insert_with(|| {
                PlayerLocation::Station(Station {
                    solarsystem,
                    station: 0,
                })
            });
        let station = if let PlayerLocation::Station(station) = location {
            station.station
        } else {
            0
        };

        match instruction {
            Instruction::ModuleUntargeted(_) | Instruction::ModuleTargeted(_) => {
                // Already handled
            }
            Instruction::Undock => {
                let ship = player_ships
                    .get(player)
                    .expect("player undocking also has to be in player_ships");
                if ship.fitting.is_valid(statics) {
                    site_entities.push(SiteEntity::Player(Player {
                        id: player.to_string(),
                        shiplayout: ship.fitting.layout.to_string(),
                    }));
                    *location = PlayerLocation::Site(site::Identifier {
                        solarsystem,
                        site_unique: Info::generate_station(solarsystem, station).site_unique,
                    });
                } else {
                    eprintln!(
                        "player tried to undock with invalid ship {} {:?}",
                        player, ship
                    );
                }
            }
            Instruction::Facility(facility) => {
                match facility.service {
                    Service::Dock => {
                        remove_player_from_entities(site_entities, player);
                        *location = PlayerLocation::Station(Station {
                            solarsystem,
                            station,
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
            Instruction::Warp(warp) => {
                if warp.site_unique != site_info.site_unique {
                    remove_player_from_entities(site_entities, player);
                    *location = PlayerLocation::Warp(Warp {
                        solarsystem,
                        towards_site_unique: warp.site_unique.to_string(),
                    });
                }
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
