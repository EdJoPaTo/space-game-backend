use space_game_typings::fixed::npc_faction::NpcFaction;
use space_game_typings::persist::player::Player;
use space_game_typings::persist::site::Site;
use space_game_typings::site::instruction::{Instruction, UseModuleTargeted};
use space_game_typings::site::Entity;

#[allow(clippy::cast_possible_truncation)]
pub fn generate(_site: Site, site_entities: &[Entity]) -> Vec<(usize, Vec<Instruction>)> {
    let mut result = Vec::new();
    for (site_index, entity) in site_entities.iter().enumerate() {
        if let Entity::Npc((faction, ship)) = entity {
            match faction {
                NpcFaction::Guards => {
                    // TODO: attack bad players
                }
                NpcFaction::Pirates => {
                    let mut instructions = Vec::new();
                    if let Some((target_index, _target_player)) = get_players(site_entities).first()
                    {
                        for module_index in 0..ship.fitting.slots_targeted.len() {
                            instructions.push(Instruction::ModuleTargeted(UseModuleTargeted {
                                target_index_in_site: *target_index as u8,
                                module_index: module_index as u8,
                            }));
                        }
                    }
                    result.push((site_index, instructions));
                }
            }
        }
    }
    result
}

fn get_players(site_entities: &[Entity]) -> Vec<(usize, Player)> {
    site_entities
        .iter()
        .enumerate()
        .filter_map(|(i, entity)| {
            if let Entity::Player((player, _)) = entity {
                Some((i, *player))
            } else {
                None
            }
        })
        .collect()
}
