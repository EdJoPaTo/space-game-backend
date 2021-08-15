use anyhow::Result;
use typings::fixed::solarsystem::Solarsystem;
use typings::frontrw::site_instruction::{self, SiteInstruction};
use typings::persist::player::Player;
use typings::persist::player_assets::PlayerStationAssets;
use typings::persist::player_location::PlayerLocation;
use typings::persist::ship::Ship;

use super::{list, read, write};

fn filename_station_assets(player: Player, solarsystem: Solarsystem, station: u8) -> String {
    format!(
        "persist/station-assets/{}/{}-{}.yaml",
        player.to_string(),
        solarsystem,
        station
    )
}
fn filename_player_location(player: Player) -> String {
    format!("persist/player-location/{}.yaml", player.to_string())
}
fn filename_player_ship(player: Player) -> String {
    format!("persist/player-ship/{}.yaml", player.to_string())
}
fn filename_instructions(player: Player) -> String {
    format!("persist/player-instructions/{}.yaml", player.to_string())
}

pub fn read_station_assets(
    player: Player,
    solarsystem: Solarsystem,
    station: u8,
) -> PlayerStationAssets {
    read(&filename_station_assets(player, solarsystem, station))
}
pub fn write_station_assets(
    player: Player,
    solarsystem: Solarsystem,
    station: u8,
    assets: &PlayerStationAssets,
) -> Result<()> {
    write(
        &filename_station_assets(player, solarsystem, station),
        assets,
    )
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

pub fn read_player_ship(player: Player) -> Ship {
    read(&filename_player_ship(player))
}
pub fn write_player_ship(player: Player, ship: &Ship) -> Result<()> {
    write(&filename_player_ship(player), ship)
}

pub fn read_player_site_instructions(player: Player) -> Vec<SiteInstruction> {
    let all: Vec<SiteInstruction> = read(&filename_instructions(player));
    site_instruction::filter_possible(&all)
}
pub fn write_player_site_instructions(
    player: Player,
    instructions: &[SiteInstruction],
) -> Result<()> {
    let possible = site_instruction::filter_possible(instructions);
    write(&filename_instructions(player), &possible)
}
pub fn add_player_site_instructions(
    player: Player,
    instructions: &[SiteInstruction],
) -> Result<()> {
    let mut all = read_player_site_instructions(player);
    for additional in instructions {
        all.push(*additional);
    }
    write_player_site_instructions(player, &all)
}
