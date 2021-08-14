use typings::persist::player::Player;
use typings::persist::site_entity::{Npc, SiteEntity};

pub fn player_pos(site_entities: &[SiteEntity], player: Player) -> Option<usize> {
    site_entities
        .iter()
        .position(|o| matches!(o, SiteEntity::Player(p) if p == &player))
}

pub fn remove_player(site_entities: &mut Vec<SiteEntity>, player: Player) {
    if let Some(index) = player_pos(site_entities, player) {
        site_entities.remove(index);
    }
}

pub fn get_mut_npc(site_entities: &mut Vec<SiteEntity>, npc_index: usize) -> &mut Npc {
    let origin_entity = site_entities
        .get_mut(npc_index)
        .expect("npc index has to be existing");
    if let SiteEntity::Npc(npc) = origin_entity {
        npc
    } else {
        panic!("index is not an npc")
    }
}

pub fn get_players(site_entities: &[SiteEntity]) -> Vec<(usize, Player)> {
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
