use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use async_std::sync::Mutex;
use async_std::task::{sleep, spawn};
use space_game_typings::fixed::Statics;

use crate::persist::Persist;

mod market;
mod site_round;
mod sites;

pub async fn start(statics: Arc<Statics>, persist: Arc<Mutex<Persist>>) -> anyhow::Result<()> {
    let mut persist_once = persist.lock_arc().await;
    once(&statics, &mut persist_once)?;

    spawn(async {
        do_loop(statics, persist).await;
    });
    Ok(())
}

async fn do_loop(statics: Arc<Statics>, persist: Arc<Mutex<Persist>>) -> ! {
    loop {
        sleep(Duration::from_secs(15)).await;
        let mut persist = persist.lock_arc().await;
        if let Err(err) = once(&statics, &mut persist) {
            eprintln!("ERROR gameloop {}", err);
        }
    }
}

// TODO: ensure players in warp warp to existing site

fn once(statics: &Statics, persist: &mut Persist) -> anyhow::Result<()> {
    let site_round_took = {
        let measure = Instant::now();
        site_round::all(statics, persist);
        measure.elapsed()
    };

    let sites_took = {
        let measure = Instant::now();
        sites::all(statics, persist).map_err(|err| anyhow!("gameloop::sites {}", err))?;
        measure.elapsed()
    };

    let market_took = {
        let measure = Instant::now();
        market::all(statics, persist).map_err(|err| anyhow!("gameloop::market {}", err))?;
        measure.elapsed()
    };

    println!(
        "gameloop::once site_round:{:?} site:{:?} market:{:?}",
        site_round_took, sites_took, market_took
    );
    Ok(())
}
