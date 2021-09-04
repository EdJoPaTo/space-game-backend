use rand::Rng;
use space_game_typings::fixed::item::Ore;
use space_game_typings::fixed::module::Targeted;
use space_game_typings::fixed::npc_faction::NpcFaction;
use space_game_typings::fixed::shiplayout::ShipLayout;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::ship::{Fitting, Ship};
use space_game_typings::site::{Entity, Site, SitesNearPlanet};

use crate::persist::Persist;

fn generate_unique(existing: &mut Vec<u8>) -> u8 {
    let mut rng = rand::thread_rng();
    loop {
        let unique = rng.gen();
        if !existing.contains(&unique) {
            existing.push(unique);
            return unique;
        }
    }
}

pub fn all(statics: &Statics, persist: &mut Persist) -> anyhow::Result<()> {
    for solarsystem in statics.solarsystems.data.keys().copied() {
        let sites = persist
            .sites
            .read_sites(solarsystem)
            .expect("init at least created gate sites");

        // Asteroid Belts
        generate_asteroid_belts(statics, persist, solarsystem, &sites)?;
        spawn_asteroid_belt_pirates(statics, persist, solarsystem, &sites)?;
    }

    Ok(())
}

fn generate_asteroid_belts(
    statics: &Statics,
    persist: &mut Persist,
    solarsystem: Solarsystem,
    sites: &SitesNearPlanet,
) -> anyhow::Result<()> {
    let planets = statics.solarsystems.get(&solarsystem).planets;
    let mut existing = sites
        .all()
        .iter()
        .filter_map(|o| {
            if let Site::AsteroidField(u) = o {
                Some(*u)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut rng = rand::thread_rng();
    for _ in existing.len()..4 {
        let planet = rng.gen_range(1..=planets);
        let site = Site::AsteroidField(generate_unique(&mut existing));
        let entities = vec![
            Entity::new_asteroid(Ore::Aromit, 25, 18),
            Entity::new_asteroid(Ore::Aromit, 40, 25),
            Entity::new_asteroid(Ore::Solmit, 15, 120),
            Entity::new_asteroid(Ore::Tormit, 10, 12),
            Entity::new_asteroid(Ore::Vesmit, 6, 4),
        ];
        persist
            .sites
            .add_site(solarsystem, planet, site, &entities)?;
    }

    Ok(())
}

fn spawn_asteroid_belt_pirates(
    statics: &Statics,
    persist: &mut Persist,
    solarsystem: Solarsystem,
    sites: &SitesNearPlanet,
) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    for site in sites.all() {
        if let Site::AsteroidField(_) = site {
            let mut entities = persist.sites.read_entities(solarsystem, site)?;

            let npc_amount = entities
                .iter()
                .filter(|o| matches!(o, Entity::Npc(_)))
                .count();

            if npc_amount == 0 && rng.gen_range(0..30) == 0 {
                let fitting = Fitting {
                    layout: ShipLayout::Hecate,
                    slots_targeted: vec![Targeted::RookieLaser],
                    slots_untargeted: vec![],
                    slots_passive: vec![],
                };
                entities.push(Entity::Npc((
                    NpcFaction::Pirates,
                    Ship::new(statics, fitting),
                )));
                persist.sites.write_entities(solarsystem, site, &entities)?;
            }
        }
    }
    Ok(())
}
