use anyhow::Result;
use space_game_typings::fixed::facility::Facility;
use space_game_typings::fixed::module::targeted::Targeted;
use space_game_typings::fixed::npc_faction::NpcFaction;
use space_game_typings::fixed::shiplayout::ShipLayout;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::{Solarsystems, Statics};
use space_game_typings::persist::ship::{Fitting, Ship};
use space_game_typings::persist::site::{Site, SitesNearPlanet};
use space_game_typings::persist::site_entity::{Npc, SiteEntity};

use super::{delete, read_meh, write};

// TODO: mutex on solarsystem for read and write access
// read needs probably public and private methods to prevent deadlock?

fn filename_site_entities(solarsystem: Solarsystem, site: Site) -> String {
    format!(
        "persist/site-entities/{}/{}.yaml",
        solarsystem,
        site.to_string()
    )
}

fn filename_sites(solarsystem: Solarsystem) -> String {
    format!("persist/sites/{}.yaml", solarsystem)
}

pub fn read_site_entities(solarsystem: Solarsystem, site: Site) -> Result<Vec<SiteEntity>> {
    read_meh(&filename_site_entities(solarsystem, site))
}

pub fn read_sites(solarsystem: Solarsystem) -> Result<SitesNearPlanet> {
    read_meh(&filename_sites(solarsystem))
}
pub fn read_sites_everywhere(solarsystems: &Solarsystems) -> Vec<(Solarsystem, Site)> {
    let mut result = Vec::new();
    for solarsystem in solarsystems.data.keys().copied() {
        let sites = read_sites(solarsystem).expect("init at least created gate sites");
        for site in sites.all() {
            result.push((solarsystem, site));
        }
    }
    result
}

pub fn write_site_entities(
    solarsystem: Solarsystem,
    site: Site,
    entities: &[SiteEntity],
) -> Result<()> {
    if entities.is_empty() {
        return Err(anyhow::anyhow!(
            "dont write empty site entities. remove_site instead {} {:?}",
            solarsystem,
            site
        ));
    }
    write(&filename_site_entities(solarsystem, site), &entities)
}

fn write_sites(solarsystem: Solarsystem, sites: &SitesNearPlanet) -> Result<()> {
    write(&filename_sites(solarsystem), sites)
}

pub fn add_site(
    solarsystem: Solarsystem,
    planet: u8,
    site: Site,
    entities: &[SiteEntity],
) -> Result<()> {
    write_site_entities(solarsystem, site, entities)?;

    let mut sites = read_sites(solarsystem)?;
    sites.add(planet, site);
    write_sites(solarsystem, &sites)
}

pub fn remove_site(solarsystem: Solarsystem, site: Site) -> Result<()> {
    let mut sites = read_sites(solarsystem)?;
    sites.remove(site);
    write_sites(solarsystem, &sites)?;
    delete(&filename_site_entities(solarsystem, site))?;
    Ok(())
}

pub fn ensure_static_sites(statics: &Statics) -> Result<()> {
    for (solarsystem, data) in &statics.solarsystems.data {
        let mut sites = read_sites(*solarsystem).unwrap_or_default();

        // Purge stations and stargates from overview.
        // If they are gone from the data players shouldnt be able to warp to them anymore
        for site in sites.all() {
            if matches!(site, Site::Stargate(_) | Site::Station(_)) {
                sites.remove(site);
            }
        }

        // Ensure stargates exist
        for (target, planet) in &data.stargates {
            let site = Site::Stargate(*target);

            // Read and purge facilities and guards
            let mut entities = read_site_entities(*solarsystem, site)
                .unwrap_or_default()
                .iter()
                .filter(|o| !matches!(o, SiteEntity::Facility(_) | SiteEntity::Npc(_)))
                .cloned()
                .collect::<Vec<_>>();
            // Add guards
            add_guards(statics, &mut entities);
            // Add stargate
            entities.insert(0, SiteEntity::Facility(Facility::Stargate));
            write_site_entities(*solarsystem, site, &entities)?;

            sites.add(*planet, site);
        }

        // Ensure stations exist
        for (index, planet) in data.stations.iter().copied().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let site = Site::Station(index as u8);

            // Read and purge facilities and guards
            let mut entities = read_site_entities(*solarsystem, site)
                .unwrap_or_default()
                .iter()
                .filter(|o| !matches!(o, SiteEntity::Facility(_) | SiteEntity::Npc(_)))
                .cloned()
                .collect::<Vec<_>>();
            // Add guards
            add_guards(statics, &mut entities);
            // Add station
            entities.insert(0, SiteEntity::Facility(Facility::Station));
            write_site_entities(*solarsystem, site, &entities)?;

            sites.add(planet, site);
        }

        write_sites(*solarsystem, &sites)?;
    }
    Ok(())
}

fn add_guards(statics: &Statics, entities: &mut Vec<SiteEntity>) {
    for _ in 0..3 {
        let fitting = Fitting {
            layout: ShipLayout::Paladin,
            slots_passive: vec![],
            slots_targeted: vec![
                Targeted::GuardianLaser,
                Targeted::GuardianLaser,
                Targeted::GuardianLaser,
                Targeted::GuardianLaser,
                Targeted::GuardianLaser,
                Targeted::GuardianLaser,
            ],
            slots_untargeted: vec![],
        };
        entities.insert(
            0,
            SiteEntity::Npc(Npc {
                faction: NpcFaction::Guards,
                ship: Ship::new(statics, fitting),
            }),
        );
    }
}
