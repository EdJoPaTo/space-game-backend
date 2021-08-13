use std::collections::HashMap;

use typings::frontrw::site_instruction::SiteInstruction;
use typings::persist::player;

// TODO: allow for npc instructions to be added and sorted into the same ordered Vec<>

pub fn sort(
    player_instructions: &HashMap<player::Identifier, Vec<SiteInstruction>>,
) -> Vec<(player::Identifier, SiteInstruction)> {
    let mut result: Vec<(player::Identifier, SiteInstruction)> = Vec::new();
    for (player, instructions) in player_instructions {
        for instruction in instructions {
            result.push((player.to_string(), instruction.clone()));
        }
    }
    result.sort_by(|a, b| a.1.cmp(&b.1));
    result
}

pub fn cleanup(player_instructions: &mut HashMap<player::Identifier, Vec<SiteInstruction>>) {
    // TODO: keep something like warp
    for (_player, instructions) in player_instructions.iter_mut() {
        instructions.clear();
    }
}

#[cfg(test)]
use typings::frontrw::site_instruction::{ModuleTargeted, ModuleUntargeted, Warp};

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
    let sorted = sort(&example);
    assert_eq!(sorted.len(), 3);
    assert_eq!(
        sorted[0],
        (
            "player1".to_string(),
            SiteInstruction::ModuleUntargeted(ModuleUntargeted { module_index: 0 })
        )
    );
    assert_eq!(
        sorted[1],
        (
            "player2".to_string(),
            SiteInstruction::ModuleTargeted(ModuleTargeted {
                module_index: 0,
                target_index_in_site: 0,
            })
        )
    );
    assert_eq!(
        sorted[2],
        (
            "player1".to_string(),
            SiteInstruction::Warp(Warp {
                site_unique: "666".to_string()
            })
        )
    );
}
