use std::collections::HashMap;

use typings::frontrw::instruction::Instruction;
use typings::persist::player;

// TODO: allow for npc instructions to be added and sorted into the same ordered Vec<>

pub fn sort(
    instructions: &HashMap<player::Identifier, Vec<Instruction>>,
) -> Vec<(player::Identifier, Instruction)> {
    let mut result: Vec<(player::Identifier, Instruction)> = Vec::new();
    for (player, instructions) in instructions {
        for instruction in instructions {
            result.push((player.to_string(), instruction.clone()));
        }
    }
    result.sort_by(|a, b| a.1.cmp(&b.1));
    result
}

#[cfg(test)]
use typings::frontrw::instruction::{ModuleTargeted, ModuleUntargeted};

#[test]
fn player_sorted_works() {
    let mut example = HashMap::new();
    example.insert(
        "player1".to_string(),
        vec![
            Instruction::Undock,
            Instruction::ModuleUntargeted(ModuleUntargeted { module_index: 0 }),
        ],
    );
    example.insert(
        "player2".to_string(),
        vec![Instruction::ModuleTargeted(ModuleTargeted {
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
            Instruction::ModuleUntargeted(ModuleUntargeted { module_index: 0 })
        )
    );
    assert_eq!(
        sorted[1],
        (
            "player2".to_string(),
            Instruction::ModuleTargeted(ModuleTargeted {
                module_index: 0,
                target_index_in_site: 0,
            })
        )
    );
    assert_eq!(sorted[2], ("player1".to_string(), Instruction::Undock));
}
