use typings::persist::site_entity::SiteEntity;

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
