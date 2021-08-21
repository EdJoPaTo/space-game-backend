use std::time::{Duration, Instant};

use async_std::task::{sleep, spawn};
use typings::fixed::Statics;

mod ship;
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

fn once(statics: &Statics) -> anyhow::Result<()> {
    let measure = Instant::now();
    site_round::all(statics)?;
    let site_round_took = measure.elapsed();

    let measure = Instant::now();
    ship::all()?;
    let ship_took = measure.elapsed();

    let measure = Instant::now();
    sites::all(statics)?;
    let sites_took = measure.elapsed();

    println!(
        "gameloop::once {:?} {:?} {:?}",
        site_round_took, ship_took, sites_took
    );
    Ok(())
}
