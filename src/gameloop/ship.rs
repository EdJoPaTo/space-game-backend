use typings::fixed::{solarsystem, Statics};
use typings::persist::player_location::{PlayerLocation, Station};
use typings::persist::ship::{Ship, Status};

use crate::persist::player::{
    read_all_player_locations, read_player_ship, write_player_location, write_player_ship,
};

/// Check all ships.
/// If dead → reset location and ship.
/// If station → fill status to max.
pub fn all(statics: &Statics) -> anyhow::Result<()> {
    for (player, location) in read_all_player_locations() {
        if let Ok(mut ship) = read_player_ship(&player) {
            if !ship.status.is_alive() {
                eprintln!("player is dead {} {:?}", player, ship);
                // TODO: use home station
                write_player_location(
                    &player,
                    &PlayerLocation::Station(Station {
                        solarsystem: solarsystem::Identifier::default(),
                        station: 0,
                    }),
                )?;
                write_player_ship(&player, &Ship::default())?;
            } else if matches!(location, PlayerLocation::Station(_)) {
                if let Some(status) = Status::new(statics, &ship.fitting) {
                    if ship.status != status {
                        // TODO: station button "repair"
                        eprintln!("repair player ship in station {}", player);
                        ship.status = status;
                        write_player_ship(&player, &ship)?;
                    }
                }
            }
        }
    }
    Ok(())
}
