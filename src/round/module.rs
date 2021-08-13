use std::collections::HashMap;

use typings::fixed::{module, Statics};
use typings::persist::player;
use typings::persist::ship::Ship;
use typings::persist::site_entity::SiteEntity;

use super::effect::{apply_to_origin, apply_to_target};

pub fn apply_untargeted(statics: &Statics, ship: &mut Ship, module_index: u8) {
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
pub fn apply_targeted_to_origin<'s>(
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

pub fn apply_targeted_to_target(
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
