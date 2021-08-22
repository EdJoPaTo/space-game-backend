use space_game_typings::persist::player_location::PlayerLocation;
use space_game_typings::persist::ship::Ship;

use crate::persist::player::{
    read_all_player_locations, read_player_ship, write_player_location, write_player_ship,
};

/// Check all ships.
/// If dead → reset location and ship.
pub fn all() -> anyhow::Result<()> {
    for (player, _location) in read_all_player_locations() {
        let ship = read_player_ship(player);
        if !ship.status.is_alive() {
            eprintln!("player is dead {:?} {:?}", player, ship);
            // TODO: use home station
            write_player_location(player, PlayerLocation::default())?;
            write_player_ship(player, &Ship::default())?;
        }
    }
    Ok(())
}
