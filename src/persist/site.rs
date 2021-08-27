use anyhow::Result;
use space_game_typings::fixed::facility::Facility;
use space_game_typings::fixed::module::targeted::Targeted;
use space_game_typings::fixed::npc_faction::NpcFaction;
use space_game_typings::fixed::shiplayout::ShipLayout;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::{Solarsystems, Statics};
use space_game_typings::ship::{Fitting, Ship};
use space_game_typings::site::{Entity, Site, SitesNearPlanet};

use super::{delete, read, read_meh, write};

fn filename_site_entities(solarsystem: Solarsystem, site: Site) -> String {
    format!(
        "persist/sites/entities/{}/{}.yaml",
        solarsystem,
        site.to_string()
    )
}
fn filename_sites(solarsystem: Solarsystem) -> String {
    format!("persist/sites/{}.yaml", solarsystem)
}
fn filename_warping(solarsystem: Solarsystem) -> String {
    format!("persist/warping/{}.yaml", solarsystem)
}

pub fn read_entitiy_warping(solarsystem: Solarsystem) -> Vec<(Site, Entity)> {
    read(&filename_warping(solarsystem))
}
pub fn pop_entity_warping(solarsystem: Solarsystem, site: Site) -> Result<Vec<Entity>> {
    let mut other = Vec::new();
    let mut result = Vec::new();
    for (towards, entity) in read_entitiy_warping(solarsystem) {
        if site == towards {
            result.push(entity);
        } else {
            other.push((towards, entity));
        }
    }
    write(filename_warping(solarsystem), &other)?;
    Ok(result)
}
pub fn add_entity_warping(solarsystem: Solarsystem, target: Site, entity: Entity) -> Result<()> {
    let mut current = read_entitiy_warping(solarsystem);
    current.push((target, entity));
    write(&filename_warping(solarsystem), &current)
}

pub fn read_site_entities(solarsystem: Solarsystem, site: Site) -> Result<Vec<Entity>> {
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
    entities: &[Entity],
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
    entities: &[Entity],
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
                .filter(|o| !matches!(o, Entity::Facility(_) | Entity::Npc(_)))
                .cloned()
                .collect::<Vec<_>>();
            // Add guards
            add_guards(statics, &mut entities);
            // Add stargate
            entities.insert(0, Entity::Facility(Facility::Stargate));
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
                .filter(|o| !matches!(o, Entity::Facility(_) | Entity::Npc(_)))
                .cloned()
                .collect::<Vec<_>>();
            // Add guards
            add_guards(statics, &mut entities);
            // Add station
            entities.insert(0, Entity::Facility(Facility::Station));
            write_site_entities(*solarsystem, site, &entities)?;

            sites.add(planet, site);
        }

        write_sites(*solarsystem, &sites)?;
    }
    Ok(())
}

fn add_guards(statics: &Statics, entities: &mut Vec<Entity>) {
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
            Entity::Npc((NpcFaction::Guards, Ship::new(statics, fitting))),
        );
    }
}
