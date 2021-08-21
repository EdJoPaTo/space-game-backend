use std::collections::HashMap;

use typings::fixed::facility::Service;
use typings::fixed::solarsystem::Solarsystem;
use typings::fixed::Statics;
use typings::frontread::site_log::{SiteLog, SiteLogActor};
use typings::frontrw::site_instruction::SiteInstruction;
use typings::persist::player::Player;
use typings::persist::player_location::PlayerLocation;
use typings::persist::ship::Ship;
use typings::persist::site::Site;
use typings::persist::site_entity::{Npc, SiteEntity};

use self::effect::apply_passives;
use self::instructions::Actor;

mod effect;
mod entities;
mod facility;
mod instructions;
mod module;
mod warp_player;

pub struct Outputs {
    pub site_log: Vec<SiteLog>,
}

#[allow(clippy::too_many_arguments)]
pub fn advance(
    statics: &Statics,
    solarsystem: Solarsystem,
    site: Site,
    site_entities: &mut Vec<SiteEntity>,
    player_instructions: &mut HashMap<Player, Vec<SiteInstruction>>,
    npc_instructions: &[(usize, Vec<SiteInstruction>)],
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player_ships: &mut HashMap<Player, Ship>,
    players_warping_in: &[Player],
) -> Outputs {
    let sorted_instructions = instructions::sort(player_instructions, npc_instructions);
    if !sorted_instructions.is_empty() {
        println!(
            "site::handle {:>15} {:?} {:?}",
            solarsystem.to_string(),
            site,
            sorted_instructions
        );
    }

    let mut site_log = Vec::new();

    for (actor, instruction) in &sorted_instructions {
        match instruction {
            SiteInstruction::ModuleUntargeted(module) => {
                module::apply_untargeted(
                    statics,
                    site_entities,
                    player_ships,
                    actor,
                    module.module_index,
                );
            }
            SiteInstruction::ModuleTargeted(module) => {
                module::apply_targeted(
                    statics,
                    site_entities,
                    player_ships,
                    actor,
                    module.module_index,
                    module.target_index_in_site,
                    &mut site_log,
                );
            }
            SiteInstruction::SelfDestruct => {
                module::self_destruct(site_entities, player_ships, actor);
            }
            SiteInstruction::Facility(facility) => {
                if let Actor::Player(player) = *actor {
                    match facility.service {
                        Service::Dock => facility::dock(
                            solarsystem,
                            site,
                            site_entities,
                            player_locations,
                            player_ships,
                            player,
                            &mut site_log,
                        ),
                        Service::Jump => facility::jump(
                            solarsystem,
                            site,
                            site_entities,
                            player_locations,
                            player_ships,
                            player,
                            &mut site_log,
                        ),
                    }
                } else {
                    panic!("only players can use facilities");
                }
            }
            SiteInstruction::Warp(warp) => {
                if let Actor::Player(player) = *actor {
                    warp_player::out(
                        solarsystem,
                        site_entities,
                        player_locations,
                        player_ships,
                        player,
                        warp.target,
                        &mut site_log,
                    );
                } else {
                    panic!("only players can warp");
                }
            }
        }
    }

    finishup_entities(statics, site_entities, player_ships, &mut site_log);

    // Add players in warp to here
    warp_player::in_site(
        solarsystem,
        site,
        site_entities,
        player_locations,
        player_ships,
        players_warping_in,
        &mut site_log,
    );

    instructions::cleanup(player_instructions);

    Outputs { site_log }
}

/// - apply passive effects
/// - ensure status is within ship layout limits
/// - cleanup dead
fn finishup_entities(
    statics: &Statics,
    site_entities: &mut Vec<SiteEntity>,
    player_ships: &mut HashMap<Player, Ship>,
    site_log: &mut Vec<SiteLog>,
) {
    let mut remaining = Vec::new();
    for entity in site_entities.iter() {
        match entity {
            SiteEntity::Facility(_) => {
                remaining.push(entity.clone());
            }
            SiteEntity::Lifeless(l) => {
                if !l.status.is_alive() {
                    site_log.push(SiteLog::RapidUnscheduledDisassembly(
                        SiteLogActor::Lifeless(l.id),
                    ));
                } else if l.is_collapsed() {
                    site_log.push(SiteLog::Collapse(SiteLogActor::Lifeless(l.id)));
                } else {
                    remaining.push(entity.clone());
                }
            }
            SiteEntity::Npc(npc) => {
                let layout = statics.ship_layouts.get(&npc.fitting.layout);
                let mut status = npc.status;
                status = apply_passives(status, &layout.round_effects);
                // Ensure the ship is within its layout limits
                let status = status.min_layout(statics, &npc.fitting);
                if status.is_alive() {
                    remaining.push(SiteEntity::Npc(Npc {
                        faction: npc.faction,
                        fitting: npc.fitting.clone(),
                        status,
                    }));
                } else {
                    site_log.push(SiteLog::RapidUnscheduledDisassembly(SiteLogActor::Npc((
                        npc.faction,
                        npc.fitting.layout,
                    ))));
                }
            }
            SiteEntity::Player(player) => {
                let ship = player_ships
                    .get_mut(player)
                    .expect("player has to be in player_ships");
                let layout = statics.ship_layouts.get(&ship.fitting.layout);
                ship.status = apply_passives(ship.status, &layout.round_effects);
                // Ensure the ship is within its layout limits
                ship.status = ship.status.min_layout(statics, &ship.fitting);
                if ship.status.is_alive() {
                    remaining.push(entity.clone());
                } else {
                    site_log.push(SiteLog::RapidUnscheduledDisassembly(SiteLogActor::Player(
                        (*player, ship.fitting.layout),
                    )));

                    // The calling function has to check if players are dead and handle that.
                    // The round logic doesnt care about that anymore.
                }
            }
        }
    }
    *site_entities = remaining;
}
