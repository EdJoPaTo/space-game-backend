use space_game_typings::fixed::item::{Item, Ore};
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::frontrw::station_instruction::StationInstruction;
use space_game_typings::player::location::{PlayerLocation, PlayerLocationSite};
use space_game_typings::player::Player;
use space_game_typings::ship::Ship;
use space_game_typings::site::{Entity, Site};

use crate::persist::player::{
    read_player_generals, read_player_location, read_station_assets, write_player_generals,
    write_player_location, write_station_assets,
};
use crate::persist::site::{read_site_entities, write_site_entities};

pub fn do_instructions(
    statics: &Statics,
    player: Player,
    instructions: &[StationInstruction],
) -> anyhow::Result<()> {
    let location = read_player_location(player);
    let solarsystem = location.solarsystem();
    let station = match location {
        PlayerLocation::Station(s) => s.station,
        PlayerLocation::Site(_) | PlayerLocation::Warp(_) => {
            return Err(anyhow::anyhow!("player is not docked"))
        }
    };
    for instruction in instructions.iter().copied() {
        do_instruction(statics, player, instruction, solarsystem, station)?;
    }
    Ok(())
}

fn do_instruction(
    statics: &Statics,
    player: Player,
    instruction: StationInstruction,
    solarsystem: Solarsystem,
    station: u8,
) -> anyhow::Result<()> {
    let mut assets = read_station_assets(player, solarsystem, station);
    match instruction {
        StationInstruction::Repair => {
            for ship in &mut assets.ships {
                let collateral = ship.fitting.maximum_collateral(statics);
                if ship.collateral != collateral {
                    eprintln!("repair player ship in station {:?}", player);
                    ship.collateral = collateral;
                }
            }
        }
        StationInstruction::Undock => {
            // TODO: undocking shouldnt be instantanious. It should also be handled with the round logic
            let ship = if let Some(ship) = assets.ships.last() {
                if let Err(err) = ship.fitting.is_valid(statics) {
                    return Err(anyhow::anyhow!(
                        "That ship wont fly {:?} {:?} {:?}",
                        player,
                        err,
                        ship
                    ));
                }

                assets.ships.pop().unwrap()
            } else {
                Ship::default()
            };

            let site = Site::Station(station);

            let mut entities = read_site_entities(solarsystem, site)?;
            entities.push(Entity::Player((player, ship)));
            write_site_entities(solarsystem, site, &entities)?;

            write_player_location(
                player,
                PlayerLocation::Site(PlayerLocationSite { solarsystem, site }),
            )?;
        }
        StationInstruction::SellOre => {
            let mut generals = read_player_generals(player);

            for ship in &mut assets.ships {
                let ore_cargo = ship
                    .cargo
                    .to_vec()
                    .iter()
                    .filter_map(|(item, amount)| match item {
                        Item::Ore(o) => Some((*o, *amount)),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                for (ore, amount) in ore_cargo {
                    let value = match ore {
                        Ore::Aromit => 300,
                        Ore::Solmit => 400,
                        Ore::Tormit => 500,
                        Ore::Vesmit => 600,
                    };
                    generals.paperclips += u64::from(amount) * value;
                    ship.cargo = ship.cargo.checked_sub(ore.into(), amount).unwrap();
                }
            }

            write_player_generals(player, &generals)?;
        }
    }
    write_station_assets(player, solarsystem, station, &assets)?;
    Ok(())
}
