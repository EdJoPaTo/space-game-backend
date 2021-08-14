use std::collections::HashMap;

use typings::fixed::npc_faction::NpcFaction;
use typings::frontrw::site_instruction::{ModuleTargeted, SiteInstruction};
use typings::persist::player::Player;
use typings::persist::site;
use typings::persist::site_entity::SiteEntity;

use super::entities;

// TODO: allow for npc instructions to be added and sorted into the same ordered Vec<>

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Player(Player),
    /// Index within the site
    Npc(usize),
}

pub fn sort(
    player_instructions: &HashMap<Player, Vec<SiteInstruction>>,
    npc_instructions: &[(usize, Vec<SiteInstruction>)],
) -> Vec<(Actor, SiteInstruction)> {
    let mut result: Vec<(Actor, SiteInstruction)> = Vec::new();
    for (player, instructions) in player_instructions {
        for instruction in instructions {
            result.push((Actor::Player(*player), instruction.clone()));
        }
    }
    for (npc, instructions) in npc_instructions {
        for instruction in instructions {
            result.push((Actor::Npc(*npc), instruction.clone()));
        }
    }
    result.sort_by(|a, b| a.1.cmp(&b.1));
    result
}

#[allow(clippy::cast_possible_truncation)]
pub fn generate_for_npc(
    _site_info: &site::Info,
    site_entities: &[SiteEntity],
) -> Vec<(usize, Vec<SiteInstruction>)> {
    let mut result = Vec::new();
    for (site_index, entity) in site_entities.iter().enumerate() {
        if let SiteEntity::Npc(npc) = entity {
            match npc.faction {
                NpcFaction::Guards => {
                    // TODO: attack bad players
                }
                NpcFaction::Pirates => {
                    let mut instructions = Vec::new();
                    if let Some((target_index, _target_player)) =
                        entities::get_players(site_entities).first()
                    {
                        for module_index in 0..npc.fitting.slots_targeted.len() {
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

pub fn cleanup(player_instructions: &mut HashMap<Player, Vec<SiteInstruction>>) {
    // TODO: keep something like warp
    for (_player, instructions) in player_instructions.iter_mut() {
        instructions.clear();
    }
}

#[test]
fn player_sorted_works() {
    let mut example = HashMap::new();
    example.insert(
        Player::Telegram(1),
        vec![
            SiteInstruction::Warp(typings::frontrw::site_instruction::Warp {
                site_unique: "666".to_string(),
            }),
            SiteInstruction::ModuleUntargeted(
                typings::frontrw::site_instruction::ModuleUntargeted { module_index: 0 },
            ),
        ],
    );
    example.insert(
        Player::Telegram(2),
        vec![SiteInstruction::ModuleTargeted(ModuleTargeted {
            module_index: 0,
            target_index_in_site: 0,
        })],
    );
    let sorted = sort(&example, &[]);
    assert_eq!(sorted.len(), 3);
    assert_eq!(
        sorted[0],
        (
            Actor::Player(Player::Telegram(1)),
            SiteInstruction::ModuleUntargeted(
                typings::frontrw::site_instruction::ModuleUntargeted { module_index: 0 }
            )
        )
    );
    assert_eq!(
        sorted[1],
        (
            Actor::Player(Player::Telegram(2)),
            SiteInstruction::ModuleTargeted(ModuleTargeted {
                module_index: 0,
                target_index_in_site: 0,
            })
        )
    );
    assert_eq!(
        sorted[2],
        (
            Actor::Player(Player::Telegram(1)),
            SiteInstruction::Warp(typings::frontrw::site_instruction::Warp {
                site_unique: "666".to_string()
            })
        )
    );
}
