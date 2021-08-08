use anyhow::Result;
use typings::fixed::npc_faction::NpcFaction;
use typings::fixed::site::Kind;
use typings::fixed::Solarsystems;
use typings::fixed::{facility, solarsystem};
use typings::persist::ship::{Fitting, Status};
use typings::persist::site::{Info, SitesNearPlanet};
use typings::persist::site_entity::{Facility, Npc, SiteEntity};

use super::{read_meh, write};

// TODO: mutex on solarsystem for read and write access
// read needs probably public and private methods to prevent deadlock?

fn filename_site_entities(solarsystem: solarsystem::Identifier, site_unique: &str) -> String {
    format!("persist/site-entities/{}/{}.yaml", solarsystem, site_unique)
}

fn filename_sites(solarsystem: solarsystem::Identifier) -> String {
    format!("persist/sites/{}.yaml", solarsystem)
}

pub fn read_site_entities(
    solarsystem: solarsystem::Identifier,
    site_unique: &str,
) -> Result<Vec<SiteEntity>> {
    read_meh(&filename_site_entities(solarsystem, site_unique))
}

pub fn read_sites(solarsystem: solarsystem::Identifier) -> Result<SitesNearPlanet> {
    read_meh(&filename_sites(solarsystem))
}
pub fn read_sites_everywhere(solarsystems: &Solarsystems) -> Vec<(solarsystem::Identifier, Info)> {
    let mut result = Vec::new();
    for solarsystem in solarsystems.keys().copied() {
        let sites = read_sites(solarsystem).expect("init at least created gate sites");
        for site in sites.values().flatten() {
            result.push((solarsystem, site.clone()));
        }
    }
    result
}

pub fn write_site_entities(
    solarsystem: solarsystem::Identifier,
    site_unique: &str,
    entities: &[SiteEntity],
) -> Result<()> {
    write(&filename_site_entities(solarsystem, site_unique), &entities)
}

fn write_sites(solarsystem: solarsystem::Identifier, sites: &SitesNearPlanet) -> Result<()> {
    write(&filename_sites(solarsystem), sites)
}

pub fn read_site_info(
    solarsystem: solarsystem::Identifier,
    site_unique: &str,
) -> Result<Option<Info>> {
    let sites = read_sites(solarsystem)?;
    let site = sites
        .values()
        .flatten()
        .find(|o| o.site_unique == site_unique);
    Ok(site.cloned())
}

pub fn add(
    solarsystem: solarsystem::Identifier,
    planet: u8,
    site: Info,
    entities: &[SiteEntity],
) -> Result<()> {
    write_site_entities(solarsystem, &site.site_unique, entities)?;

    let mut sites = read_sites(solarsystem)?;
    sites.entry(planet).or_default().push(site);
    write_sites(solarsystem, &sites)
}

pub fn remove(solarsystem: solarsystem::Identifier, site_unique: &str) -> Result<()> {
    let mut sites = read_sites(solarsystem)?;
    if let Some((planet, index)) = position_of_site_unique(&sites, site_unique) {
        let sites = sites.get_mut(&planet).unwrap();
        sites.remove(index);
    }

    write_sites(solarsystem, &sites)
}

fn position_of_site_unique(sites: &SitesNearPlanet, unique: &str) -> Option<(u8, usize)> {
    for (planet, site_info) in sites {
        if let Some(position) = site_info.iter().position(|o| o.site_unique == unique) {
            return Some((planet.to_owned(), position));
        }
    }
    None
}

pub fn ensure_statics(solarsystems: &Solarsystems) -> Result<()> {
    for (solarsystem, data) in solarsystems {
        let mut sites = read_sites(*solarsystem).unwrap_or_default();

        // Purge stations and stargates from overview.
        // If they are gone from the data players shouldnt be able to warp to them anymore
        for planet in 1..=data.planets {
            let sites = sites.entry(planet).or_default();
            *sites = sites
                .iter()
                .filter(|o| !matches!(o.kind, Kind::Stargate | Kind::Station))
                .cloned()
                .collect();
        }

        // Ensure stargates exist
        for (target, planet) in &data.stargates {
            let name = target.to_string();
            let site_unique = format!("stargate{}", target);

            // Read and purge facilities and guards
            let mut entities = read_site_entities(*solarsystem, &site_unique)
                .unwrap_or_default()
                .iter()
                .filter(|o| !matches!(o, SiteEntity::Facility(_) | SiteEntity::Npc(_)))
                .cloned()
                .collect::<Vec<_>>();
            // Add guards
            add_guards(&mut entities);
            // Add stargate
            entities.insert(
                0,
                SiteEntity::Facility(Facility {
                    id: facility::Identifier::Stargate,
                }),
            );
            write_site_entities(*solarsystem, &site_unique, &entities)?;

            sites.entry(*planet).or_default().insert(
                0,
                Info {
                    kind: Kind::Stargate,
                    name: Some(name),
                    site_unique,
                },
            );
        }

        // Ensure stations exist
        for (index, planet) in data.stations.iter().copied().enumerate() {
            let number = index + 1;
            let name = format!("{} {}", solarsystem, number);
            let site_unique = format!("station{}", number);

            // Read and purge facilities and guards
            let mut entities = read_site_entities(*solarsystem, &site_unique)
                .unwrap_or_default()
                .iter()
                .filter(|o| !matches!(o, SiteEntity::Facility(_) | SiteEntity::Npc(_)))
                .cloned()
                .collect::<Vec<_>>();
            // Add guards
            add_guards(&mut entities);
            // Add station
            entities.insert(
                0,
                SiteEntity::Facility(Facility {
                    id: facility::Identifier::Station,
                }),
            );
            write_site_entities(*solarsystem, &site_unique, &entities)?;

            sites.entry(planet).or_default().insert(
                0,
                Info {
                    kind: Kind::Station,
                    name: Some(name),
                    site_unique,
                },
            );
        }

        write_sites(*solarsystem, &sites)?;
    }
    Ok(())
}

fn add_guards(entities: &mut Vec<SiteEntity>) {
    for _ in 0..3 {
        entities.insert(
            0,
            SiteEntity::Npc(Npc {
                faction: NpcFaction::Guards,
                fitting: Fitting {
                    // TODO: ensure layout exists
                    layout: "shiplayoutFrigate".to_string(),
                    slots_passive: vec![],
                    slots_targeted: vec![],
                    slots_untargeted: vec![],
                },
                // TODO: better status
                status: Status {
                    capacitor: 42,
                    hitpoints_armor: 42,
                    hitpoints_structure: 42,
                },
            }),
        );
    }
}
