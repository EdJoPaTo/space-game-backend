use std::collections::HashMap;

use typings::fixed::npc_faction::NpcFaction;
use typings::frontrw::site_instruction::SiteInstruction;
use typings::persist::site_entity::SiteEntity;
use typings::persist::{player, site};

#[cfg(test)]
use typings::frontrw::site_instruction::{ModuleTargeted, ModuleUntargeted, Warp};

// TODO: allow for npc instructions to be added and sorted into the same ordered Vec<>

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    Player(player::Identifier),
    /// Index within the site
    Npc(usize),
}

pub fn sort(
    player_instructions: &HashMap<player::Identifier, Vec<SiteInstruction>>,
    npc_instructions: &[(usize, Vec<SiteInstruction>)],
) -> Vec<(Actor, SiteInstruction)> {
    let mut result: Vec<(Actor, SiteInstruction)> = Vec::new();
    for (player, instructions) in player_instructions {
        for instruction in instructions {
            result.push((Actor::Player(player.to_string()), instruction.clone()));
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

pub fn generate_for_npc(
    _site_info: &site::Info,
    site_entities: &[SiteEntity],
) -> Vec<(usize, Vec<SiteInstruction>)> {
    let result = Vec::new();
    for (_index, entity) in site_entities.iter().enumerate() {
        if let SiteEntity::Npc(npc) = entity {
            #[allow(clippy::match_same_arms)]
            match npc.faction {
                NpcFaction::Guards => {
                    // TODO: attack bad players
                }
                NpcFaction::Pirates => {
                    // TODO: attack random player
                }
            }
        }
    }
    result
}

pub fn cleanup(player_instructions: &mut HashMap<player::Identifier, Vec<SiteInstruction>>) {
    // TODO: keep something like warp
    for (_player, instructions) in player_instructions.iter_mut() {
        instructions.clear();
    }
}

#[test]
fn player_sorted_works() {
    let mut example = HashMap::new();
    example.insert(
        "player1".to_string(),
        vec![
            SiteInstruction::Warp(Warp {
                site_unique: "666".to_string(),
            }),
            SiteInstruction::ModuleUntargeted(ModuleUntargeted { module_index: 0 }),
        ],
    );
    example.insert(
        "player2".to_string(),
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
            Actor::Player("player1".to_string()),
            SiteInstruction::ModuleUntargeted(ModuleUntargeted { module_index: 0 })
        )
    );
    assert_eq!(
        sorted[1],
        (
            Actor::Player("player2".to_string()),
            SiteInstruction::ModuleTargeted(ModuleTargeted {
                module_index: 0,
                target_index_in_site: 0,
            })
        )
    );
    assert_eq!(
        sorted[2],
        (
            Actor::Player("player1".to_string()),
            SiteInstruction::Warp(Warp {
                site_unique: "666".to_string()
            })
        )
    );
}
