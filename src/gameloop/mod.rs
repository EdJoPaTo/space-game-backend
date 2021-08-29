use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use async_std::task::{sleep, spawn};
use space_game_typings::fixed::Statics;

use crate::persist::Persist;

mod site_round;
mod sites;

pub async fn start(statics: Arc<Statics>, persist: Persist) -> anyhow::Result<()> {
    once(&statics, &persist).await?;
    spawn(async {
        do_loop(statics, persist).await;
    });
    Ok(())
}

async fn do_loop(statics: Arc<Statics>, persist: Persist) -> ! {
    loop {
        sleep(Duration::from_secs(15)).await;
        if let Err(err) = once(&statics, &persist).await {
            eprintln!("ERROR gameloop {}", err);
        }
    }
}

// TODO: ensure players in warp warp to existing site

async fn once(statics: &Statics, persist: &Persist) -> anyhow::Result<()> {
    let site_round_took = {
        let measure = Instant::now();
        site_round::all(statics);
        measure.elapsed()
    };

    let sites_took = {
        let measure = Instant::now();
        sites::all(statics).map_err(|err| anyhow!("gameloop::sites {}", err))?;
        measure.elapsed()
    };

    let market_took = {
        let measure = Instant::now();
        let market = persist.market.lock_arc().await;
        let trades = market
            .trade()
            .map_err(|err| anyhow!("gameloop::market {}", err))?;
        // TODO: notify about trades
        if !trades.is_empty() {
            println!("trades {} {:?}", trades.len(), trades);
        }
        measure.elapsed()
    };

    println!(
        "gameloop::once site_round:{:?} site:{:?} market:{:?}",
        site_round_took, sites_took, market_took
    );
    Ok(())
}
