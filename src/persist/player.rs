use anyhow::Result;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::player::location::PlayerLocation;
use space_game_typings::player::{General, Player, StationAssets};
use space_game_typings::site::instruction::{filter_possible, Instruction};
use space_game_typings::site::Log;

use super::{delete, list, read, write};

fn filename_station_assets(player: Player, solarsystem: Solarsystem, station: u8) -> String {
    format!(
        "persist/station-assets/{}/{}-{}.yaml",
        player.to_string(),
        solarsystem,
        station
    )
}
fn filename_player_generals(player: Player) -> String {
    format!("persist/player-generals/{}.yaml", player.to_string())
}
fn filename_player_location(player: Player) -> String {
    format!("persist/player-location/{}.yaml", player.to_string())
}
fn filename_instructions(player: Player) -> String {
    format!("persist/player-instructions/{}.yaml", player.to_string())
}
fn filename_site_log(player: Player) -> String {
    format!("persist/player-sitelog/{}.yaml", player.to_string())
}

pub fn read_station_assets(player: Player, solarsystem: Solarsystem, station: u8) -> StationAssets {
    read(&filename_station_assets(player, solarsystem, station))
}
pub fn write_station_assets(
    player: Player,
    solarsystem: Solarsystem,
    station: u8,
    assets: &StationAssets,
) -> Result<()> {
    write(
        &filename_station_assets(player, solarsystem, station),
        assets,
    )
}

pub fn read_player_generals(player: Player) -> General {
    read(&filename_player_generals(player))
}
pub fn write_player_generals(player: Player, generals: &General) -> Result<()> {
    write(&filename_player_generals(player), generals)
}

pub fn read_player_location(player: Player) -> PlayerLocation {
    read(&filename_player_location(player))
}
pub fn write_player_location(player: Player, location: PlayerLocation) -> Result<()> {
    write(&filename_player_location(player), &location)
}
pub fn read_all_player_locations() -> Vec<(Player, PlayerLocation)> {
    let list = list("persist/player-location/");
    let list = list
        .iter()
        .filter_map(|o| o.file_stem())
        .filter_map(std::ffi::OsStr::to_str)
        .filter_map(|o| o.parse().ok());
    let mut result = Vec::new();
    for player in list {
        let location = read_player_location(player);
        result.push((player, location));
    }
    result
}

pub fn read_player_site_instructions(player: Player) -> Vec<Instruction> {
    let all: Vec<Instruction> = read(&filename_instructions(player));
    filter_possible(&all)
}
pub fn write_player_site_instructions(player: Player, instructions: &[Instruction]) -> Result<()> {
    let possible = filter_possible(instructions);
    write(&filename_instructions(player), &possible)
}
pub fn add_player_site_instructions(player: Player, instructions: &[Instruction]) -> Result<()> {
    let mut all = read_player_site_instructions(player);
    for additional in instructions {
        all.push(*additional);
    }
    write_player_site_instructions(player, &all)
}

fn read_player_site_log(player: Player) -> Vec<Log> {
    read(&filename_site_log(player))
}
fn write_player_site_log(player: Player, site_log: &[Log]) -> Result<()> {
    write(filename_site_log(player), &site_log)
}
pub fn add_player_site_log(player: Player, site_log: &[Log]) -> Result<()> {
    let mut all = read_player_site_log(player);
    for additional in site_log {
        all.push(*additional);
    }
    write_player_site_log(player, &all)
}
pub fn pop_player_site_log(player: Player) -> Result<Vec<Log>> {
    let log = read_player_site_log(player);
    delete(&filename_site_log(player))?;
    Ok(log)
}
pub fn list_players_with_site_log() -> Vec<Player> {
    list("persist/player-sitelog/")
        .iter()
        .filter_map(|o| o.file_stem())
        .filter_map(std::ffi::OsStr::to_str)
        .filter_map(|o| o.parse().ok())
        .collect()
}
