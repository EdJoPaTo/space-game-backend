use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::frontrw::station_instruction::StationInstruction;
use space_game_typings::persist::player::Player;
use space_game_typings::persist::player_location::{PlayerLocation, PlayerLocationSite};
use space_game_typings::persist::site::Site;
use space_game_typings::persist::site_entity::SiteEntity;

use crate::persist::player::{
    read_player_generals, read_player_location, read_player_ship, write_player_generals,
    write_player_location, write_player_ship,
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
    match instruction {
        StationInstruction::Repair => {
            let mut ship = read_player_ship(player);
            let status = ship.fitting.maximum_status(statics);
            if ship.status != status {
                eprintln!("repair player ship in station {:?}", player);
                ship.status = status;
                write_player_ship(player, &ship)?;
            }
        }
        StationInstruction::Undock => {
            let ship = read_player_ship(player);
            if let Err(err) = ship.fitting.is_valid(statics) {
                return Err(anyhow::anyhow!(
                    "That ship wont fly {:?} {:?} {:?}",
                    player,
                    err,
                    ship
                ));
            }

            let site = Site::Station(station);

            let mut entities = read_site_entities(solarsystem, site)?;
            entities.push(SiteEntity::Player(player));
            write_site_entities(solarsystem, site, &entities)?;

            write_player_location(
                player,
                PlayerLocation::Site(PlayerLocationSite { solarsystem, site }),
            )?;
        }
        StationInstruction::SellOre => {
            let mut generals = read_player_generals(player);
            let mut ship = read_player_ship(player);
            let ore = ship.cargo.ore;

            generals.paperclips += u64::from(ore) * 500;
            ship.cargo.ore = 0;

            write_player_generals(player, &generals)?;
            write_player_ship(player, &ship)?;
        }
    }

    Ok(())
}
