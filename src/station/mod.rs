use space_game_typings::fixed::item::Item;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::player::location::{PlayerLocation, PlayerLocationSite};
use space_game_typings::player::Player;
use space_game_typings::ship::Ship;
use space_game_typings::site::{Entity, Site};
use space_game_typings::station::instruction::Instruction;
use space_game_typings::storage::Storage;

use crate::persist::site::{read_site_entities, write_site_entities};
use crate::persist::Persist;

pub fn do_instructions(
    statics: &Statics,
    persist: &mut Persist,
    player: Player,
    instructions: &[Instruction],
) -> anyhow::Result<()> {
    let location = persist.player_locations.read(player);
    let solarsystem = location.solarsystem();
    let station = match location {
        PlayerLocation::Station(s) => s.station,
        PlayerLocation::Site(_) | PlayerLocation::Warp(_) => {
            return Err(anyhow::anyhow!("player is not docked"))
        }
    };
    for instruction in instructions.iter().copied() {
        do_instruction(statics, persist, player, instruction, solarsystem, station)?;
    }
    Ok(())
}

fn do_instruction(
    statics: &Statics,
    persist: &mut Persist,
    player: Player,
    instruction: Instruction,
    solarsystem: Solarsystem,
    station: u8,
) -> anyhow::Result<()> {
    let mut assets = persist
        .player_station_assets
        .read(player, solarsystem, station);
    match instruction {
        Instruction::SwitchShip(index) => {
            assets.switch_ship(index);
        }
        Instruction::Repair => {
            if let Some(ship) = &mut assets.current_ship {
                let collateral = ship.fitting.maximum_collateral(statics);
                if ship.collateral != collateral {
                    eprintln!("repair player ship in station {:?}", player);
                    ship.collateral = collateral;
                }
            }
        }
        Instruction::Undock => {
            // TODO: undocking shouldnt be instantanious. It should also be handled with the round logic
            let ship = assets.current_ship.take().unwrap_or_default();
            let site = Site::Station(station);
            let mut entities = read_site_entities(solarsystem, site)?;
            entities.push(Entity::Player((player, ship)));
            write_site_entities(solarsystem, site, &entities)?;
            persist.player_locations.write(
                player,
                PlayerLocation::Site(PlayerLocationSite { solarsystem, site }),
            )?;
        }
        Instruction::ShipCargoLoad(i) => {
            if assets.current_ship.is_none() {
                assets.current_ship = Some(Ship::default());
            }
            let ship = assets.current_ship.as_mut().unwrap();
            let free = ship.free_cargo(statics);
            let amount = free.min(i.amount);
            let amount = assets.storage.take_max(i.item, amount);
            ship.cargo.saturating_add(i.item, amount);
        }
        Instruction::ShipCargoUnload(i) => {
            if let Some(ship) = &mut assets.current_ship {
                let amount = ship.cargo.take_max(i.item, i.amount);
                assets.storage.saturating_add(i.item, amount);
            }
        }
        Instruction::Buy(o) => {
            let (item, order) = o.to_order(player, solarsystem, station);
            let mut general = persist.player_generals.read(player);
            if let Some(remaining) = general.paperclips.checked_sub(order.total_paperclips()) {
                general.paperclips = remaining;
                persist.market.buy(item, order)?;
                persist.player_generals.write(player, &general)?;
            } else {
                return Err(anyhow::anyhow!("not enough money for buy order"));
            }
        }
        Instruction::Sell(o) => {
            let (item, order) = o.to_order(player, solarsystem, station);
            if assets.storage.take_exact(item, order.amount) {
                persist.market.sell(item, order)?;
            } else {
                return Err(anyhow::anyhow!("not enough items for sell order"));
            }
        }
        Instruction::Recycle { item, amount } => {
            recycle(statics, &mut assets.storage, item, amount);
        }
    }
    persist
        .player_station_assets
        .write(player, solarsystem, station, &assets)?;
    Ok(())
}

fn recycle<I: Into<Item>>(statics: &Statics, storage: &mut Storage, item: I, amount: u32) {
    let item = item.into();
    let amount = storage.take_max(item, amount);
    for (mineral, i) in &statics.items.get(&item).recycle {
        storage.saturating_add(*mineral, amount.saturating_mul(*i));
    }
}

#[test]
fn recycle_nothing() {
    use space_game_typings::fixed::module::Passive;
    let statics = Statics::default();
    let mut storage = Storage::new_empty();
    let expected = Storage::new_empty();
    recycle(&statics, &mut storage, Passive::RookieArmorPlate, 2);
    assert_eq!(storage.to_vec(), expected.to_vec());
}

#[test]
fn recycle_something() {
    use space_game_typings::fixed::item::Mineral;
    use space_game_typings::fixed::module::Passive;
    let statics = Statics::default();
    let mut storage = Storage::new_single(Passive::RookieArmorPlate, 2);
    let expected = Storage::new_single(Mineral::Derite, 2);
    recycle(&statics, &mut storage, Passive::RookieArmorPlate, 2);
    assert_eq!(storage.to_vec(), expected.to_vec());
}
