use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::player::location::{PlayerLocation, PlayerLocationSite};
use space_game_typings::player::Player;
use space_game_typings::ship::Ship;
use space_game_typings::site::{Entity, Site};
use space_game_typings::station::instruction::Instruction;

use crate::persist::player::{
    read_player_location, read_station_assets, write_player_location, write_station_assets,
};
use crate::persist::site::{read_site_entities, write_site_entities};
use crate::persist::Persist;

pub async fn do_instructions(
    statics: &Statics,
    persist: &Persist,
    player: Player,
    instructions: &[Instruction],
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
        do_instruction(statics, persist, player, instruction, solarsystem, station).await?;
    }
    Ok(())
}

async fn do_instruction(
    statics: &Statics,
    persist: &Persist,
    player: Player,
    instruction: Instruction,
    solarsystem: Solarsystem,
    station: u8,
) -> anyhow::Result<()> {
    let mut assets = read_station_assets(player, solarsystem, station);
    match instruction {
        Instruction::Repair => {
            for ship in &mut assets.ships {
                let collateral = ship.fitting.maximum_collateral(statics);
                if ship.collateral != collateral {
                    eprintln!("repair player ship in station {:?}", player);
                    ship.collateral = collateral;
                }
            }
        }
        Instruction::Undock => {
            // TODO: undocking shouldnt be instantanious. It should also be handled with the round logic
            let ship = if let Some(ship) = assets.ships.last() {
                if let Err(err) = ship.fitting.is_valid(statics) {
                    return Err(anyhow::anyhow!(
                        "That ship wont fly {:?} {:?} {:?}",
                        player,
                        err,
                        ship
                    ));
                }

                assets.ships.pop().unwrap()
            } else {
                Ship::default()
            };

            let site = Site::Station(station);

            let mut entities = read_site_entities(solarsystem, site)?;
            entities.push(Entity::Player((player, ship)));
            write_site_entities(solarsystem, site, &entities)?;

            write_player_location(
                player,
                PlayerLocation::Site(PlayerLocationSite { solarsystem, site }),
            )?;
        }
        Instruction::Buy(o) => {
            let (item, order) = o.to_order(player, solarsystem, station);
            let market = persist.market.lock_arc().await;
            market.buy(item, order)?;
        }
        Instruction::Sell(o) => {
            let (item, order) = o.to_order(player, solarsystem, station);
            let market = persist.market.lock_arc().await;
            market.sell(item, order)?;
        }
    }
    write_station_assets(player, solarsystem, station, &assets)?;
    Ok(())
}
