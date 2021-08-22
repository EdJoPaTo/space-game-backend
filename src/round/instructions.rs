use std::collections::HashMap;

use space_game_typings::frontrw::site_instruction::{self, SiteInstruction};
use space_game_typings::persist::player::Player;

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
        for instruction in site_instruction::filter_possible(instructions) {
            result.push((Actor::Player(*player), instruction));
        }
    }
    for (npc, instructions) in npc_instructions {
        for instruction in instructions {
            result.push((Actor::Npc(*npc), *instruction));
        }
    }
    result.sort_by(|a, b| a.1.cmp(&b.1));
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
        vec![SiteInstruction::Warp(site_instruction::Warp {
            target: space_game_typings::persist::site::Site::Station(42),
        })],
    );
    example.insert(
        Player::Telegram(2),
        vec![
            SiteInstruction::ModuleTargeted(site_instruction::ModuleTargeted {
                module_index: 0,
                target_index_in_site: 0,
            }),
            SiteInstruction::ModuleUntargeted(site_instruction::ModuleUntargeted {
                module_index: 0,
            }),
        ],
    );
    let sorted = sort(&example, &[]);
    assert_eq!(sorted.len(), 3);
    assert_eq!(
        sorted[0],
        (
            Actor::Player(Player::Telegram(2)),
            SiteInstruction::ModuleUntargeted(site_instruction::ModuleUntargeted {
                module_index: 0
            })
        )
    );
    assert_eq!(
        sorted[1],
        (
            Actor::Player(Player::Telegram(2)),
            SiteInstruction::ModuleTargeted(site_instruction::ModuleTargeted {
                module_index: 0,
                target_index_in_site: 0,
            })
        )
    );
    assert_eq!(
        sorted[2],
        (
            Actor::Player(Player::Telegram(1)),
            SiteInstruction::Warp(site_instruction::Warp {
                target: space_game_typings::persist::site::Site::Station(42),
            })
        )
    );
}
