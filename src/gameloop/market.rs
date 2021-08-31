use rand::Rng;
use space_game_typings::fixed::item::{Item, Ore};
use space_game_typings::fixed::npc_faction::NpcFaction;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::market::{Order, Trader};

use crate::persist::{Market, Persist};

pub fn all(statics: &Statics, persist: &mut Persist) -> anyhow::Result<()> {
    let assets = &mut persist.player_station_assets;
    let generals = &mut persist.player_generals;
    let market = &mut persist.market;
    let notifications = &mut persist.player_notifications;

    for (item, trade) in market.trade()? {
        println!("trade happened {:?} {:?}", item, trade);

        // Give player the goods
        if let Trader::Player(player) = trade.buyer {
            let mut current = assets.read(player, trade.solarsystem, trade.station);
            current.storage.saturating_add(item, trade.amount);
            assets.write(player, trade.solarsystem, trade.station, &current)?;
        }

        if let Trader::Player(player) = trade.seller {
            let mut current = generals.read(player);
            current.paperclips = current.paperclips.saturating_add(trade.total_paperclips());
            generals.write(player, &current)?;
        }

        // Notify about trade
        if let Trader::Player(player) = trade.buyer {
            notifications.add(player, (item, trade))?;
        }
        if let Trader::Player(player) = trade.seller {
            notifications.add(player, (item, trade))?;
        }
    }

    generate_ore_orders(statics, market)?;

    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
fn generate_ore_orders(statics: &Statics, market: &mut Market) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    let trader = Trader::Npc(NpcFaction::Guards);

    let ore = Ore::Aromit;
    let orders = get_npc_buy_orders(market, ore);
    for (solarsystem, details) in &statics.solarsystems.data {
        for station in 0..details.stations.len() as u8 {
            let remaining_amount: u32 = orders
                .iter()
                .filter(|o| o.solarsystem == *solarsystem && o.station == station)
                .map(|o| o.amount)
                .sum();
            if remaining_amount < 800 {
                market.buy(
                    ore.into(),
                    Order::new_now(*solarsystem, station, trader, 1000, 200),
                )?;
            }
        }
    }

    let ore = Ore::Solmit;
    let orders = get_npc_buy_orders(market, ore);
    for (solarsystem, details) in &statics.solarsystems.data {
        let remaining_amount: u32 = orders
            .iter()
            .filter(|o| o.solarsystem == *solarsystem)
            .map(|o| o.amount)
            .sum();
        if remaining_amount < 500 {
            let stations = details.stations.len() as u8;
            let station = rng.gen_range(0..stations);
            market.buy(
                ore.into(),
                Order::new_now(*solarsystem, station, trader, 800, 350),
            )?;
        }
    }

    let ore = Ore::Tormit;
    let orders = get_npc_buy_orders(market, ore);
    for (solarsystem, details) in &statics.solarsystems.data {
        let remaining_amount: u32 = orders
            .iter()
            .filter(|o| o.solarsystem == *solarsystem)
            .map(|o| o.amount)
            .sum();
        if remaining_amount < 300 {
            let stations = details.stations.len() as u8;
            let station = rng.gen_range(0..stations);
            market.buy(
                ore.into(),
                Order::new_now(*solarsystem, station, trader, 500, 450),
            )?;
        }
    }

    let ore = Ore::Vesmit;
    let orders = get_npc_buy_orders(market, ore);
    let remaining_amount: u32 = orders.iter().map(|o| o.amount).sum();
    if remaining_amount < 100 {
        market.buy(
            ore.into(),
            Order::new_now(Solarsystem::Vosu, 0, trader, 200, 950),
        )?;
    }

    Ok(())
}

fn get_npc_buy_orders<I: Into<Item>>(market: &Market, item: I) -> Vec<Order> {
    market
        .get(item.into())
        .buy
        .iter()
        .filter(|o| matches!(o.trader, Trader::Npc(_)))
        .copied()
        .collect()
}
