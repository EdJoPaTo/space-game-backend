use std::time::{Duration, Instant};

use anyhow::anyhow;
use async_std::task::{sleep, spawn};
use space_game_typings::fixed::Statics;

mod site_round;
mod sites;

pub fn start(statics: &Statics) -> anyhow::Result<()> {
    once(statics)?;
    spawn(async {
        do_loop().await;
    });
    Ok(())
}

async fn do_loop() -> ! {
    let statics = Statics::default();
    loop {
        sleep(Duration::from_secs(15)).await;
        if let Err(err) = once(&statics) {
            eprintln!("ERROR gameloop {}", err);
        }
    }
}

// TODO: ensure players in warp warp to existing site

fn once(statics: &Statics) -> anyhow::Result<()> {
    let measure = Instant::now();
    site_round::all(statics).map_err(|err| anyhow!("gameloop::site_round {}", err))?;
    let site_round_took = measure.elapsed();

    let measure = Instant::now();
    sites::all(statics).map_err(|err| anyhow!("gameloop::sites {}", err))?;
    let sites_took = measure.elapsed();

    println!("gameloop::once {:?} {:?}", site_round_took, sites_took);
    Ok(())
}
