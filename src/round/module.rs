use std::collections::HashMap;

use typings::fixed::module::targeted::Targeted;
use typings::fixed::round_effect::RoundEffect;
use typings::fixed::{module, Statics};
use typings::frontread::site_log::{SiteLog, SiteLogActor};
use typings::persist::player::Player;
use typings::persist::ship::{Cargo, CargoAmounts, Fitting, Ship, Status};
use typings::persist::site_entity::SiteEntity;

use super::effect::{apply_to_origin, apply_to_target};
use super::entities;
use super::instructions::Actor;

pub fn apply_untargeted(
    statics: &Statics,
    site_entities: &mut Vec<SiteEntity>,
    player_ships: &mut HashMap<Player, Ship>,
    actor: &Actor,
    module_index: u8,
) {
    let (fitting, status) = match actor {
        Actor::Player(player) => {
            let ship = player_ships
                .get_mut(player)
                .expect("player_ships has to contain player with instructions");
            (&ship.fitting, &mut ship.status)
        }
        Actor::Npc(npc_index) => {
            let npc = entities::get_mut_npc(site_entities, *npc_index);
            (&npc.fitting, &mut npc.status)
        }
    };
    if let Some(module) = fitting
        .slots_untargeted
        .get(module_index as usize)
        .map(|o| statics.modules_untargeted.get(o))
    {
        if let Some(my_new_status) = apply_to_origin(*status, &module.effects) {
            *status = my_new_status;
        }
    } else {
        println!(
            "WARN: untargeted module not handled {} {:?} {:?}",
            module_index, fitting, status
        );
    }
}

pub fn apply_targeted(
    statics: &Statics,
    site_entities: &mut Vec<SiteEntity>,
    player_ships: &mut HashMap<Player, Ship>,
    actor: &Actor,
    module_index: u8,
    target_index_in_site: u8,
    site_log: &mut Vec<SiteLog>,
) {
    let (site_log_origin, fitting, status, free_cargo) = match actor {
        Actor::Player(player) => {
            let ship = player_ships
                .get_mut(player)
                .expect("player_ships has to contain player with instructions");
            let log_actor = SiteLogActor::Player((*player, ship.fitting.layout));
            let free_cargo = ship.cargo.free(statics, &ship.fitting);
            (log_actor, &ship.fitting, &mut ship.status, free_cargo)
        }
        Actor::Npc(npc_index) => {
            let npc = entities::get_mut_npc(site_entities, *npc_index);
            let log_actor = SiteLogActor::Npc((npc.faction, npc.fitting.layout));
            let free_cargo = npc.cargo.free(statics, &npc.fitting);
            (log_actor, &npc.fitting, &mut npc.status, free_cargo)
        }
    };
    let loot = if let Some((targeted, details)) =
        apply_targeted_to_origin(statics, fitting, status, module_index)
    {
        #[allow(clippy::option_if_let_else)]
        if let Some(target) = site_entities.get_mut(target_index_in_site as usize) {
            let loot = apply_targeted_to_target(player_ships, target, details, free_cargo);

            let site_log_target = SiteLogActor::from(player_ships, target);
            site_log.push(SiteLog::ModuleTargeted((
                site_log_origin,
                targeted,
                site_log_target,
            )));
            loot
        } else {
            Cargo::default()
        }
    } else {
        Cargo::default()
    };

    match actor {
        Actor::Player(player) => {
            let ship = player_ships
                .get_mut(player)
                .expect("player_ships has to contain player with instructions");
            ship.cargo = ship.cargo.add(&loot);
        }
        Actor::Npc(npc_index) => {
            let npc = entities::get_mut_npc(site_entities, *npc_index);
            npc.cargo = npc.cargo.add(&loot);
        }
    }
}

#[must_use]
fn apply_targeted_to_origin<'s>(
    statics: &'s Statics,
    origin_fitting: &Fitting,
    origin_status: &mut Status,
    module_index: u8,
) -> Option<(Targeted, &'s module::targeted::Details)> {
    if let Some(targeted) = origin_fitting.slots_targeted.get(module_index as usize) {
        let details = statics.modules_targeted.get(targeted);
        if let Some(origin_new_status) = apply_to_origin(*origin_status, &details.effects_origin) {
            *origin_status = origin_new_status;
            return Some((*targeted, details));
        }
    } else {
        println!(
            "WARN: player targeted module not handled {} {:?} {:?}",
            module_index, origin_fitting, origin_status
        );
    }
    None
}

#[must_use]
/// Returns the loot
fn apply_targeted_to_target(
    player_ships: &mut HashMap<Player, Ship>,
    target: &mut SiteEntity,
    module: &module::targeted::Details,
    free_cargo: CargoAmounts,
) -> Cargo {
    match target {
        SiteEntity::Facility(_) => {
            // Currently immune
            Cargo::default()
        }
        SiteEntity::Lifeless(l) => {
            l.status = apply_to_target(l.status, &module.effects_target);

            let ore = module
                .effects_target
                .iter()
                .find_map(|o| match o {
                    RoundEffect::Mine(amount) => Some(*amount),
                    _ => None,
                })
                .map_or(0, |mining_strength| {
                    let amount = mining_strength.min(l.remaining_ore).min(free_cargo.ore);
                    l.remaining_ore -= amount;
                    amount
                });

            Cargo { ore }
        }
        SiteEntity::Npc(npc) => {
            npc.status = apply_to_target(npc.status, &module.effects_target);
            Cargo::default()
        }
        SiteEntity::Player(player) => {
            let target_ship = player_ships
                .get_mut(player)
                .expect("player in site has to be in player_ships");
            target_ship.status = apply_to_target(target_ship.status, &module.effects_target);
            Cargo::default()
        }
    }
}

pub fn self_destruct(
    site_entities: &mut Vec<SiteEntity>,
    player_ships: &mut HashMap<Player, Ship>,
    actor: &Actor,
) {
    match actor {
        Actor::Player(player) => {
            let ship = player_ships
                .get_mut(player)
                .expect("player in site has to be in player_ships");
            ship.status = Status::DEAD;
        }
        Actor::Npc(npc_index) => {
            let npc = entities::get_mut_npc(site_entities, *npc_index);
            npc.status = Status::DEAD;
        }
    }
    // No need to add this to the site log. It logs the dead ship anyway.
}
