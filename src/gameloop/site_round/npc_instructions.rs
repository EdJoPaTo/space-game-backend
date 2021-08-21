use typings::fixed::npc_faction::NpcFaction;
use typings::frontrw::site_instruction::{ModuleTargeted, SiteInstruction};
use typings::persist::player::Player;
use typings::persist::site::Site;
use typings::persist::site_entity::SiteEntity;

#[allow(clippy::cast_possible_truncation)]
pub fn generate(_site: Site, site_entities: &[SiteEntity]) -> Vec<(usize, Vec<SiteInstruction>)> {
    let mut result = Vec::new();
    for (site_index, entity) in site_entities.iter().enumerate() {
        if let SiteEntity::Npc(npc) = entity {
            match npc.faction {
                NpcFaction::Guards => {
                    // TODO: attack bad players
                }
                NpcFaction::Pirates => {
                    let mut instructions = Vec::new();
                    if let Some((target_index, _target_player)) = get_players(site_entities).first()
                    {
                        for module_index in 0..npc.ship.fitting.slots_targeted.len() {
                            instructions.push(SiteInstruction::ModuleTargeted(ModuleTargeted {
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

fn get_players(site_entities: &[SiteEntity]) -> Vec<(usize, Player)> {
    site_entities
        .iter()
        .enumerate()
        .filter_map(|(i, entity)| {
            if let SiteEntity::Player(player) = entity {
                Some((i, *player))
            } else {
                None
            }
        })
        .collect()
}
