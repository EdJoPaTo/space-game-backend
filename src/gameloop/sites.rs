use rand::Rng;
use space_game_typings::fixed::lifeless::Lifeless;
use space_game_typings::fixed::module::targeted::Targeted;
use space_game_typings::fixed::npc_faction::NpcFaction;
use space_game_typings::fixed::shiplayout::ShipLayout;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::ship::{Fitting, Ship};
use space_game_typings::site::{Entity, EntityLifeless, Site, SitesNearPlanet};

use crate::persist::site::{read_site_entities, read_sites, write_site_entities};

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

pub fn all(statics: &Statics) -> anyhow::Result<()> {
    for solarsystem in statics.solarsystems.data.keys().copied() {
        let sites = read_sites(solarsystem).expect("init at least created gate sites");

        // Asteroid Belts
        generate_asteroid_belts(statics, solarsystem, &sites)?;
        spawn_asteroid_belt_pirates(statics, solarsystem, &sites)?;
    }

    Ok(())
}

fn generate_asteroid_belts(
    statics: &Statics,
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
        let mut entities = Vec::new();
        for _ in 0..5 {
            entities.push(Entity::Lifeless(EntityLifeless::new(
                &statics.lifeless,
                Lifeless::Asteroid,
            )));
        }
        crate::persist::site::add_site(solarsystem, planet, site, &entities)?;
    }

    Ok(())
}

fn spawn_asteroid_belt_pirates(
    statics: &Statics,
    solarsystem: Solarsystem,
    sites: &SitesNearPlanet,
) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    for site in sites.all() {
        if let Site::AsteroidField(_) = site {
            let mut entities = read_site_entities(solarsystem, site)?;

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
                write_site_entities(solarsystem, site, &entities)?;
            }
        }
    }
    Ok(())
}
