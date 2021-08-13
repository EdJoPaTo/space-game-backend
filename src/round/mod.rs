use std::collections::HashMap;

use typings::fixed::facility::Service;
use typings::fixed::module;
use typings::fixed::solarsystem::Solarsystem;
use typings::fixed::Statics;
use typings::frontrw::site_instruction::SiteInstruction;
use typings::persist::player;
use typings::persist::player_location::{PlayerLocation, Station, Warp};
use typings::persist::ship::Ship;
use typings::persist::site::{self, Info};
use typings::persist::site_entity::{Npc, Player, SiteEntity};

use effect::{apply_to_origin, apply_to_target};

mod effect;
mod site_instructions;

pub struct Outputs {}

#[allow(clippy::too_many_arguments)]
pub fn advance(
    statics: &Statics,
    solarsystem: Solarsystem,
    site_info: &site::Info,
    site_entities: &mut Vec<SiteEntity>,
    instructions: &mut HashMap<player::Identifier, Vec<SiteInstruction>>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player_ships: &mut HashMap<player::Identifier, Ship>,
    players_warping_in: &[player::Identifier],
) -> Outputs {
    // TODO: npcs need instructions too…
    // TODO: some instructions are standalone. Warp and nothing else for example. Idea: dont allow warp when some effect is there

    let sorted_instructions = site_instructions::sort(instructions);
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

        match instruction {
            SiteInstruction::ModuleUntargeted(module) => {
                apply_untargeted_module(statics, origin_ship, module.module_index);
            }
            SiteInstruction::ModuleTargeted(module) => {
                if let Some(target) = site_entities.get_mut(module.target_index_in_site as usize) {
                    if let Some(m) =
                        apply_targeted_module_to_origin(statics, origin_ship, module.module_index)
                    {
                        apply_targeted_module_to_target(player_ships, target, m);
                    }
                }
            }
            SiteInstruction::Facility(facility) => {
                // TODO: ensure still alive
                match facility.service {
                    Service::Dock => facility_dock(
                        solarsystem,
                        site_info,
                        site_entities,
                        player_locations,
                        player,
                    ),
                    Service::Jump => facility_jump(
                        solarsystem,
                        site_info,
                        site_entities,
                        player_locations,
                        player,
                    ),
                }
            }
            SiteInstruction::Warp(warp) => {
                // TODO: ensure still alive
                warp_player_out(
                    solarsystem,
                    site_entities,
                    player_locations,
                    player,
                    &warp.site_unique,
                );
            }
        }
    }

    *site_entities = finishup_entities(statics, site_entities, player_ships);

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

    Outputs {}
}

fn apply_untargeted_module(statics: &Statics, ship: &mut Ship, module_index: u8) {
    if let Some(module) = ship
        .fitting
        .slots_untargeted
        .get(module_index as usize)
        .map(|o| statics.modules_untargeted.get(o))
    {
        if let Some(my_new_status) = apply_to_origin(ship.status, &module.effects) {
            ship.status = my_new_status;
        }
    } else {
        println!(
            "WARN: untargeted module not handled {} {:?}",
            module_index, ship
        );
    }
}

#[must_use]
fn apply_targeted_module_to_origin<'s>(
    statics: &'s Statics,
    origin_ship: &mut Ship,
    module_index: u8,
) -> Option<&'s module::targeted::Details> {
    if let Some(module) = origin_ship
        .fitting
        .slots_targeted
        .get(module_index as usize)
        .map(|o| statics.modules_targeted.get(o))
    {
        if let Some(origin_new_status) = apply_to_origin(origin_ship.status, &module.effects_origin)
        {
            origin_ship.status = origin_new_status;
            return Some(module);
        }
    } else {
        println!(
            "WARN: player targeted module not handled {} {:?}",
            module_index, origin_ship
        );
    }
    None
}

fn apply_targeted_module_to_target(
    player_ships: &mut HashMap<player::Identifier, Ship>,
    target: &mut SiteEntity,
    module: &module::targeted::Details,
) {
    match target {
        SiteEntity::Facility(_) => { /* Currently immune */ }
        SiteEntity::Lifeless(l) => {
            l.status = apply_to_target(l.status, &module.effects_target);
        }
        SiteEntity::Npc(npc) => {
            npc.status = apply_to_target(npc.status, &module.effects_target);
        }
        SiteEntity::Player(player) => {
            let target_ship = player_ships
                .get_mut(&player.id)
                .expect("player in site has to be in player_ships");
            target_ship.status = apply_to_target(target_ship.status, &module.effects_target);
        }
    }
}

fn facility_jump(
    solarsystem: Solarsystem,
    site_info: &site::Info,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player: &str,
) {
    let target_solarsystem = site_info
        .site_unique
        .trim_start_matches("stargate")
        .parse()
        .unwrap_or_else(|_| {
            panic!(
                "stargate site_unique is formatted differently than expected {}",
                site_info.site_unique
            );
        });
    let target_site = Info::generate_stargate(solarsystem);
    remove_player_from_entities(site_entities, player);
    player_locations.insert(
        player.to_string(),
        PlayerLocation::Warp(Warp {
            solarsystem: target_solarsystem,
            towards_site_unique: target_site.site_unique,
        }),
    );
}

fn facility_dock(
    solarsystem: Solarsystem,
    _site_info: &site::Info,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player: &str,
) {
    remove_player_from_entities(site_entities, player);
    player_locations.insert(
        player.to_string(),
        PlayerLocation::Station(Station {
            solarsystem,
            // TODO: dock at correct station
            station: 0,
        }),
    );
}

fn warp_player_out(
    solarsystem: Solarsystem,
    site_entities: &mut Vec<SiteEntity>,
    player_locations: &mut HashMap<player::Identifier, PlayerLocation>,
    player: &str,
    target_site_unique: &str,
) {
    // TODO: ensure still alive
    remove_player_from_entities(site_entities, player);
    player_locations.insert(
        player.to_string(),
        PlayerLocation::Warp(Warp {
            solarsystem,
            towards_site_unique: target_site_unique.to_string(),
        }),
    );
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
/// - apply passive effects
/// - ensure status is within ship layout limits
/// - cleanup dead
fn finishup_entities(
    statics: &Statics,
    before: &[SiteEntity],
    player_ships: &mut HashMap<player::Identifier, Ship>,
) -> Vec<SiteEntity> {
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
                let layout = statics.ship_layouts.get(&npc.fitting.layout);
                let mut status = npc.status;
                // Apply ship passives
                if let Some(new_status) = apply_to_origin(status, &layout.round_effects) {
                    status = new_status;
                }
                // Ensure the ship is within its layout limits
                let status = status.min_layout(statics, &npc.fitting);
                if status.is_alive() {
                    remaining.push(SiteEntity::Npc(Npc {
                        faction: npc.faction,
                        fitting: npc.fitting.clone(),
                        status,
                    }));
                }
            }
            SiteEntity::Player(p) => {
                let ship = player_ships
                    .get_mut(&p.id)
                    .expect("player has to be in player_ships");
                let layout = statics.ship_layouts.get(&ship.fitting.layout);
                // Apply ship passives
                if let Some(new_status) = apply_to_origin(ship.status, &layout.round_effects) {
                    ship.status = new_status;
                }
                // Ensure the ship is within its layout limits
                ship.status = ship.status.min_layout(statics, &ship.fitting);
                if ship.status.is_alive() {
                    remaining.push(entity.clone());
                }
                // When dead another job will clean that up. The round itself doesnt care anymore about the player.
            }
        }
    }
    remaining
}
