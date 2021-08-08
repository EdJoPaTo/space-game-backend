use anyhow::Result;
use typings::fixed::solarsystem;
use typings::frontrw::instruction::Instruction;
use typings::persist::player;
use typings::persist::player_assets::PlayerStationAssets;
use typings::persist::player_location::PlayerLocation;
use typings::persist::ship::Ship;

use super::{delete, read, write};

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
fn filename_players_in_warp(solarsystem: solarsystem::Identifier, site_unique: &str) -> String {
    format!("persist/warp-towards/{}/{}.yaml", solarsystem, site_unique)
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

pub fn read_player_ship(player: &str) -> Result<Ship> {
    read(&filename_player_ship(player))
}
pub fn write_player_ship(player: &str, ship: &Ship) -> Result<()> {
    write(&filename_player_ship(player), ship)
}

pub fn pop_players_in_warp(
    solarsystem: solarsystem::Identifier,
    site_unique: &str,
) -> Vec<player::Identifier> {
    let filename = filename_players_in_warp(solarsystem, site_unique);
    let result = read(&filename).unwrap_or_default();
    delete(&filename).expect("failed to delete players in warp file");
    result
}
pub fn add_player_in_warp(
    solarsystem: solarsystem::Identifier,
    site_unique: &str,
    player: player::Identifier,
) -> Result<()> {
    let filename = filename_players_in_warp(solarsystem, site_unique);
    let mut list: Vec<player::Identifier> = read(&filename).unwrap_or_default();
    list.push(player);
    write(&filename, &list)
}

pub fn read_player_instructions(player: &str) -> Vec<Instruction> {
    read(&filename_instructions(player)).unwrap_or_default()
}
pub fn write_player_instructions(player: &str, instructions: &[Instruction]) -> Result<()> {
    write(&filename_instructions(player), &instructions)
}
