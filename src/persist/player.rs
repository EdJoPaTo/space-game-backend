use anyhow::Result;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::player::location::PlayerLocation;
use space_game_typings::player::{General, Player, StationAssets};
use space_game_typings::site::instruction::{filter_possible, Instruction};

use super::{list, read, write};

pub struct PlayerGenerals {}
impl PlayerGenerals {
    pub fn read(&self, player: Player) -> General {
        super::read(&filename_player_generals(player))
    }
    pub fn write(&mut self, player: Player, general: &General) -> Result<()> {
        super::write(&filename_player_generals(player), general)
    }
}

pub struct PlayerStationAssets {}
impl PlayerStationAssets {
    pub fn read(&self, player: Player, solarsystem: Solarsystem, station: u8) -> StationAssets {
        super::read(&filename_station_assets(player, solarsystem, station))
    }
    pub fn write(
        &mut self,
        player: Player,
        solarsystem: Solarsystem,
        station: u8,
        assets: &StationAssets,
    ) -> Result<()> {
        super::write(
            &filename_station_assets(player, solarsystem, station),
            assets,
        )
    }
}

pub struct PlayerLocations {}
impl PlayerLocations {
    pub fn read(&self, player: Player) -> PlayerLocation {
        super::read(&filename_player_location(player))
    }
    pub fn write(&mut self, player: Player, location: PlayerLocation) -> Result<()> {
        super::write(&filename_player_location(player), &location)
    }
    pub fn read_all_players(&self) -> Vec<Player> {
        list("persist/player-location/")
            .iter()
            .filter_map(|o| o.file_stem())
            .filter_map(std::ffi::OsStr::to_str)
            .filter_map(|o| o.parse().ok())
            .collect()
    }
    pub fn read_all(&self) -> Vec<(Player, PlayerLocation)> {
        let mut result = Vec::new();
        for player in self.read_all_players() {
            let location = self.read(player);
            result.push((player, location));
        }
        result
    }
}

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
