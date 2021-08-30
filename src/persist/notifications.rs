use anyhow::Result;
use space_game_typings::player::{self, Player};

pub struct Notifications {}

impl Notifications {
    fn filename(player: Player) -> String {
        format!("persist/player-notifications/{}.yaml", player.to_string())
    }

    fn read(&self, player: Player) -> player::Notifications {
        super::read(Self::filename(player))
    }

    fn write(&mut self, player: Player, notifications: &player::Notifications) -> Result<()> {
        super::write(Self::filename(player), notifications)
    }

    pub fn add<N: Into<player::Notifications>>(&mut self, player: Player, add: N) -> Result<()> {
        let mut add = add.into();
        let mut current = self.read(player);
        current.append(&mut add);
        self.write(player, &current)?;
        Ok(())
    }

    pub fn pop(&mut self, player: Player) -> Result<player::Notifications> {
        let result = self.read(player);
        super::delete(Self::filename(player))?;
        Ok(result)
    }

    pub fn list_players(&self) -> Vec<Player> {
        super::list("persist/player-notifications/")
            .iter()
            .filter_map(|o| o.file_stem())
            .filter_map(std::ffi::OsStr::to_str)
            .filter_map(|o| o.parse().ok())
            .collect()
    }
}
