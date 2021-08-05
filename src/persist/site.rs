use anyhow::Result;
use typings::fixed::facility;
use typings::fixed::npc_faction::NpcFaction;
use typings::fixed::site::Kind;
use typings::fixed::Solarsystems;
use typings::persist::ship::{Fitting, Ship, Status};
use typings::persist::site::{Info, SitesNearPlanet};
use typings::persist::site_entity::{Facility, Npc, SiteEntity};

use super::{read, write};

// TODO: mutex on solarsystem for read and write access
// read needs probably public and private methods to prevent deadlock?

fn filename_site_entries(solarsystem: &str, site_unique: &str) -> String {
    format!("persist/site-entries/{}/{}.yaml", solarsystem, site_unique)
}

fn filename_sites(solarsystem: &str) -> String {
    format!("persist/sites/{}.yaml", solarsystem)
}

pub fn read_site_entries(solarsystem: &str, site_unique: &str) -> Result<Vec<SiteEntity>> {
    read(&filename_site_entries(solarsystem, site_unique))
}

pub fn read_sites(solarsystem: &str) -> Result<SitesNearPlanet> {
    read(&filename_sites(solarsystem))
}

// write_site_entries and update are the same currently
pub fn update(solarsystem: &str, site_unique: &str, entries: &[SiteEntity]) -> Result<()> {
    write(&filename_site_entries(solarsystem, site_unique), &entries)
}

fn write_sites(solarsystem: &str, sites: &SitesNearPlanet) -> Result<()> {
    write(&filename_sites(solarsystem), sites)
}

pub fn read_site_info(solarsystem: &str, site_unique: &str) -> Result<Option<Info>> {
    let sites = read_sites(solarsystem)?;
    let site = sites.values().flatten().find(|o| o.unique == site_unique);
    Ok(site.cloned())
}

pub fn add(solarsystem: &str, planet: u8, site: Info, entries: &[SiteEntity]) -> Result<()> {
    update(solarsystem, &site.unique, entries)?;

    let mut sites = read_sites(solarsystem)?;
    sites.entry(planet).or_default().push(site);
    write_sites(solarsystem, &sites)
}

pub fn remove(solarsystem: &str, site_unique: &str) -> Result<()> {
    let mut sites = read_sites(solarsystem)?;
    if let Some((planet, index)) = position_of_site_unique(&sites, site_unique) {
        let sites = sites.get_mut(&planet).unwrap();
        sites.remove(index);
    }

    write_sites(solarsystem, &sites)
}

fn position_of_site_unique(sites: &SitesNearPlanet, unique: &str) -> Option<(u8, usize)> {
    for (planet, entries) in sites {
        if let Some(position) = entries.iter().position(|o| o.unique == unique) {
            return Some((planet.to_owned(), position));
        }
    }
    None
}

pub fn ensure_statics(solarsystems: &Solarsystems) -> Result<()> {
    for (solarsystem, data) in solarsystems {
        let mut sites = read_sites(solarsystem).unwrap_or_default();

        // Purge stations and stargates from overview.
        // If they are gone from the data players shouldnt be able to warp to them anymore
        for planet in 1..=data.planets {
            let sites = sites.entry(planet).or_default();
            *sites = sites
                .iter()
                .filter(|o| !matches!(o.kind, Kind::FacilityStargate | Kind::FacilityStation))
                .cloned()
                .collect();
        }

        // Ensure stargates exist
        for (target, planet) in &data.stargates {
            let name = target.to_string();
            let unique = format!("stargate{}", target);

            // Read and purge facilities and guards
            let mut entities = read_site_entries(solarsystem, &unique)
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
            update(solarsystem, &unique, &entities)?;

            sites.entry(*planet).or_default().insert(
                0,
                Info {
                    kind: Kind::FacilityStargate,
                    name: Some(name),
                    unique,
                },
            );
        }

        // Ensure stations exist
        for (index, planet) in data.stations.iter().copied().enumerate() {
            let number = index + 1;
            let name = format!("{} {}", solarsystem, number);
            let unique = format!("station{}", number);

            // Read and purge facilities and guards
            let mut entities = read_site_entries(solarsystem, &unique)
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
            update(solarsystem, &unique, &entities)?;

            sites.entry(planet).or_default().insert(
                0,
                Info {
                    kind: Kind::FacilityStation,
                    name: Some(name),
                    unique,
                },
            );
        }

        write_sites(solarsystem, &sites)?;
    }
    Ok(())
}

fn add_guards(entities: &mut Vec<SiteEntity>) {
    for _ in 0..3 {
        entities.insert(
            0,
            SiteEntity::Npc(Npc {
                faction: NpcFaction::Guards,
                ship: Ship {
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
                },
            }),
        );
    }
}
