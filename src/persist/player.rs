use anyhow::Result;
use typings::fixed::solarsystem;
use typings::frontrw::site_instruction::SiteInstruction;
use typings::persist::player;
use typings::persist::player_assets::PlayerStationAssets;
use typings::persist::player_location::PlayerLocation;
use typings::persist::ship::Ship;

use super::{list, read, write};

fn filename_station_assets(
    player: &str,
    solarsystem: solarsystem::Identifier,
    station: u8,
) -> String {
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
fn filename_instructions(player: &str) -> String {
    format!("persist/player-instructions/{}.yaml", player)
}

pub fn read_station_assets(
    player: &str,
    solarsystem: solarsystem::Identifier,
    station: u8,
) -> PlayerStationAssets {
    read(&filename_station_assets(player, solarsystem, station)).unwrap_or_default()
}
pub fn write_station_assets(
    player: &str,
    solarsystem: solarsystem::Identifier,
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
pub fn read_all_player_locations() -> Vec<(player::Identifier, PlayerLocation)> {
    let list = list("persist/player-location/");
    let list = list
        .iter()
        .filter_map(|o| o.file_stem())
        .filter_map(std::ffi::OsStr::to_str);
    let mut result = Vec::new();
    for player in list {
        let location = read_player_location(player).unwrap();
        result.push((player.to_string(), location));
    }
    result
}

pub fn read_player_ship(player: &str) -> Result<Ship> {
    read(&filename_player_ship(player))
}
pub fn write_player_ship(player: &str, ship: &Ship) -> Result<()> {
    write(&filename_player_ship(player), ship)
}

pub fn read_player_site_instructions(player: &str) -> Vec<SiteInstruction> {
    read(&filename_instructions(player)).unwrap_or_default()
}
pub fn write_player_site_instructions(
    player: &str,
    instructions: &[SiteInstruction],
) -> Result<()> {
    write(&filename_instructions(player), &instructions)
}
