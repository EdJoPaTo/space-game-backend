use anyhow::Result;
use typings::persist::player_assets::PlayerStationAssets;
use typings::persist::player_location::PlayerLocation;
use typings::persist::ship::Ship;

use super::{read, write};

fn filename_station_assets(player: &str, solarsystem: &str, station: u8) -> String {
    format!(
        "persist/station-assets/{}/{}-{}.yaml",
        player, solarsystem, station
    )
}
fn filename_player_location(player: &str) -> String {
    format!("persist/player-location/{}.yaml", player)
}
fn filename_player_ship(player: &str) -> String {
    format!("persist/player-ship/{}.yaml", player)
}

pub fn read_station_assets(
    player: &str,
    solarsystem: &str,
    station: u8,
) -> Result<PlayerStationAssets> {
    read(&filename_station_assets(player, solarsystem, station))
}
pub fn write_station_assets(
    player: &str,
    solarsystem: &str,
    station: u8,
    assets: &PlayerStationAssets,
) -> Result<()> {
    write(
        &filename_station_assets(player, solarsystem, station),
        assets,
    )
}

pub fn read_player_location(player: &str) -> Result<PlayerLocation> {
    read(&filename_player_location(player))
}
pub fn write_player_location(player: &str, location: &PlayerLocation) -> Result<()> {
    write(&filename_player_location(player), location)
}

pub fn read_player_ship(player: &str) -> Result<Ship> {
    read(&filename_player_ship(player))
}
pub fn write_player_ship(player: &str, ship: &Ship) -> Result<()> {
    write(&filename_player_ship(player), ship)
}
