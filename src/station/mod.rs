use typings::fixed::{solarsystem, Statics};
use typings::frontrw::station_instruction::StationInstruction;
use typings::persist::player_location::PlayerLocation;
use typings::persist::ship::Status;
use typings::persist::site::{self, Info};
use typings::persist::site_entity::{Player, SiteEntity};

use crate::persist::player::{
    read_player_location, read_player_ship, write_player_location, write_player_ship,
};
use crate::persist::site::{read_site_entities, write_site_entities};

pub fn do_instructions(
    statics: &Statics,
    player: &str,
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
    for instruction in instructions {
        do_instruction(statics, player, instruction, solarsystem, station)?;
    }
    Ok(())
}

fn do_instruction(
    statics: &Statics,
    player: &str,
    instruction: &StationInstruction,
    solarsystem: solarsystem::Identifier,
    station: u8,
) -> anyhow::Result<()> {
    match instruction {
        StationInstruction::Repair => {
            let mut ship = read_player_ship(player);
            if let Some(status) = Status::new(statics, &ship.fitting) {
                if ship.status != status {
                    eprintln!("repair player ship in station {}", player);
                    ship.status = status;
                    write_player_ship(player, &ship)?;
                }
            }
        }
        StationInstruction::Undock => {
            let ship = read_player_ship(player);
            let is_valid = ship.fitting.is_valid(statics);
            if !is_valid {
                return Err(anyhow::anyhow!("That ship wont fly {} {:?}", player, ship));
            }

            let site_unique = Info::generate_station(solarsystem, station).site_unique;

            let mut entities = read_site_entities(solarsystem, &site_unique)?;
            entities.push(SiteEntity::Player(Player {
                id: player.to_string(),
            }));
            write_site_entities(solarsystem, &site_unique, &entities)?;

            write_player_location(
                player,
                &PlayerLocation::Site(site::Identifier {
                    solarsystem,
                    site_unique,
                }),
            )?;
        }
    }

    Ok(())
}
