use rand::Rng;
use typings::fixed::lifeless::Lifeless;
use typings::fixed::module::targeted::Targeted;
use typings::fixed::npc_faction::NpcFaction;
use typings::fixed::shiplayout::ShipLayout;
use typings::fixed::site::Kind;
use typings::fixed::solarsystem::Solarsystem;
use typings::fixed::Statics;
use typings::persist::ship::{Fitting, Status};
use typings::persist::site::{Info, SitesNearPlanet};
use typings::persist::site_entity::{self, Npc, SiteEntity};

use crate::persist::site::{read_site_entities, read_sites, write_site_entities};

fn generate_unique(existing: &mut Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    loop {
        let rand_string: String = [
            rng.gen_range('A'..'Z'),
            rng.gen_range('A'..'Z'),
            rng.gen_range('0'..'9'),
            rng.gen_range('0'..'9'),
            rng.gen_range('0'..'9'),
        ]
        .iter()
        .collect();
        if !existing.contains(&rand_string) {
            existing.push(rand_string.to_string());
            return rand_string;
        }
    }
}

pub fn all(statics: &Statics) -> anyhow::Result<()> {
    for solarsystem in statics.solarsystems.data.keys().copied() {
        let sites = read_sites(solarsystem).expect("init at least created gate sites");
        let mut site_uniques = sites
            .values()
            .flatten()
            .map(|o| o.site_unique.to_string())
            .collect::<Vec<_>>();

        // Asteroid Belts
        generate_asteroid_belts(statics, solarsystem, &mut site_uniques, &sites)?;
        spawn_asteroid_belt_pirates(statics, solarsystem, &sites)?;
    }

    Ok(())
}

fn generate_asteroid_belts(
    statics: &Statics,
    solarsystem: Solarsystem,
    site_uniques: &mut Vec<String>,
    sites: &SitesNearPlanet,
) -> anyhow::Result<()> {
    let planets = statics.solarsystems.get(&solarsystem).planets;
    let amount = sites
        .values()
        .flatten()
        .filter(|o| matches!(o.kind, Kind::AsteroidField))
        .count();
    let mut rng = rand::thread_rng();
    for _ in amount..4 {
        let planet = rng.gen_range(1..=planets);
        let site = Info {
            kind: Kind::AsteroidField,
            site_unique: generate_unique(site_uniques),
            name: None,
        };
        let mut entities = Vec::new();
        for _ in 0..5 {
            entities.push(SiteEntity::Lifeless(site_entity::Lifeless {
                id: Lifeless::Asteroid,
                // TODO: use statics
                status: Status {
                    capacitor: 0,
                    hitpoints_armor: 0,
                    hitpoints_structure: 42,
                },
            }));
        }
        crate::persist::site::add(solarsystem, planet, site, &entities)?;
    }

    Ok(())
}

fn spawn_asteroid_belt_pirates(
    statics: &Statics,
    solarsystem: Solarsystem,
    sites: &SitesNearPlanet,
) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    for site in sites.values().flatten() {
        if let Kind::AsteroidField = site.kind {
            let mut entities = read_site_entities(solarsystem, &site.site_unique)?;

            let npc_amount = entities
                .iter()
                .filter(|o| matches!(o, SiteEntity::Npc(_)))
                .count();

            if npc_amount == 0 && rng.gen_range(0..30) == 0 {
                let fitting = Fitting {
                    layout: ShipLayout::Hecate,
                    slots_targeted: vec![Targeted::RookieLaser],
                    slots_untargeted: vec![],
                    slots_passive: vec![],
                };
                let status = fitting.maximum_status(statics);
                entities.push(SiteEntity::Npc(Npc {
                    faction: NpcFaction::Pirates,
                    fitting,
                    status,
                }));
                write_site_entities(solarsystem, &site.site_unique, &entities)?;
            }
        }
    }
    Ok(())
}
