use std::collections::HashMap;

use typings::fixed::facility::Service;
use typings::fixed::solarsystem::Solarsystem;
use typings::fixed::Statics;
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

pub struct Outputs {}

#[allow(clippy::too_many_arguments)]
pub fn advance(
    statics: &Statics,
    solarsystem: Solarsystem,
    site: Site,
    site_entities: &mut Vec<SiteEntity>,
    player_instructions: &mut HashMap<Player, Vec<SiteInstruction>>,
    player_locations: &mut HashMap<Player, PlayerLocation>,
    player_ships: &mut HashMap<Player, Ship>,
    players_warping_in: &[Player],
) -> Outputs {
    // TODO: some instructions are standalone. Warp and nothing else for example. Idea: dont allow warp when some effect is there

    let sorted_instructions = instructions::sort(
        player_instructions,
        &instructions::generate_for_npc(site, site_entities),
    );
    if !sorted_instructions.is_empty() {
        println!(
            "site::handle {:>15} {:?} {:?}",
            solarsystem.to_string(),
            site,
            sorted_instructions
        );
    }

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
                );
            }
            SiteInstruction::Facility(facility) => {
                if let Actor::Player(player) = *actor {
                    // TODO: ensure still alive
                    match facility.service {
                        Service::Dock => facility::dock(
                            solarsystem,
                            site,
                            site_entities,
                            player_locations,
                            player,
                        ),
                        Service::Jump => facility::jump(
                            solarsystem,
                            site,
                            site_entities,
                            player_locations,
                            player,
                        ),
                    }
                } else {
                    panic!("only players can use facilities");
                }
            }
            SiteInstruction::Warp(warp) => {
                if let Actor::Player(player) = *actor {
                    // TODO: ensure still alive
                    warp_player::out(
                        solarsystem,
                        site_entities,
                        player_locations,
                        player,
                        warp.target,
                    );
                } else {
                    panic!("only players can warp");
                }
            }
        }
    }

    *site_entities = finishup_entities(statics, site_entities, player_ships);

    // Add players in warp to here
    warp_player::in_site(
        solarsystem,
        site,
        site_entities,
        player_locations,
        players_warping_in,
    );

    instructions::cleanup(player_instructions);

    Outputs {}
}

/// - apply passive effects
/// - ensure status is within ship layout limits
/// - cleanup dead
fn finishup_entities(
    statics: &Statics,
    before: &[SiteEntity],
    player_ships: &mut HashMap<Player, Ship>,
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
                status = apply_passives(status, &layout.round_effects);
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
                }
                // When dead another job will clean that up. The round itself doesnt care anymore about the player.
            }
        }
    }
    remaining
}
