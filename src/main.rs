#![forbid(unsafe_code)]

use std::sync::Arc;
use std::time::Instant;

use async_std::sync::Mutex;
use space_game_typings::fixed::Statics;

mod gameloop;
mod persist;
mod station;
mod webserver;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let app = {
        println!("load static data...");
        let measure = Instant::now();
        let statics = Arc::new(Statics::default());
        println!("  took {:?}", measure.elapsed());

        println!("load persist data...");
        let measure = Instant::now();
        let persist = Arc::new(Mutex::new(persist::Persist::default()));
        println!("  took {:?}", measure.elapsed());

        println!("persist ensure_statics...");
        let measure = Instant::now();
        persist::ensure_static_sites(&statics).unwrap();
        println!("  took {:?}", measure.elapsed());

        println!("persist ensure_player_locations...");
        let measure = Instant::now();
        persist::ensure_player_locations(&statics).unwrap();
        println!("  took {:?}", measure.elapsed());

        println!("init webserver...");
        let app_state = webserver::State {
            statics: statics.clone(),
            persist: persist.clone(),
        };
        let measure = Instant::now();
        let app = webserver::init(app_state);
        println!("  took {:?}", measure.elapsed());

        println!("start gameloop...");
        let measure = Instant::now();
        gameloop::start(statics, persist)
            .await
            .expect("first gameloop iteration failed");
        println!("  took {:?}", measure.elapsed());

        app
    };

    println!("Starting to listen on http://localhost:8080");
    app.listen("[::]:8080").await?; // Works for both IPv4 and IPv6
    Ok(())
}
