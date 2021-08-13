use std::time::Duration;

use async_std::task::{sleep, spawn};
use typings::fixed::Statics;

mod ship;
mod site;
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
        sleep(Duration::from_secs(4)).await;
        if let Err(err) = once(&statics) {
            eprintln!("ERROR gameloop {}", err);
        }
    }
}

fn once(statics: &Statics) -> anyhow::Result<()> {
    site::all(statics)?;
    ship::all()?;
    sites::all(statics)?;
    Ok(())
}
