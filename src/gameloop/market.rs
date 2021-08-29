use rand::Rng;
use space_game_typings::fixed::item::{Item, Ore};
use space_game_typings::fixed::npc_faction::NpcFaction;
use space_game_typings::fixed::solarsystem::Solarsystem;
use space_game_typings::fixed::Statics;
use space_game_typings::market::{Order, Trader};

use crate::persist::{Market, Persist};

pub async fn all(statics: &Statics, persist: &Persist) -> anyhow::Result<()> {
    let market = persist.market().await;
    let trades = market.trade()?;
    // TODO: notify about trades
    if !trades.is_empty() {
        println!("trades {} {:?}", trades.len(), trades);
    }

    // TODO: generate npc item orders

    generate_ore_orders(statics, &market)?;

    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
fn generate_ore_orders(statics: &Statics, market: &Market) -> anyhow::Result<()> {
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
