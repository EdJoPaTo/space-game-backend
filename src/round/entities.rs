use typings::persist::site_entity::{Npc, SiteEntity};

pub fn player_pos(site_entities: &[SiteEntity], player: &str) -> Option<usize> {
    site_entities
        .iter()
        .position(|o| matches!(o, SiteEntity::Player(p) if p.id == player))
}

pub fn remove_player(site_entities: &mut Vec<SiteEntity>, player: &str) {
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
