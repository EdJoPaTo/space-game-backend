use std::collections::HashMap;
use std::time::Instant;

use math::ship::calc_max;
use persist::player::{read_player_ship, read_station_assets};
use tide::http::mime;
use tide::utils::After;
use tide::{Request, Response, StatusCode};
use typings::fixed::{solarsystem, Statics};
use typings::frontrw::instruction::Instruction;
use typings::persist::player_location::PlayerLocation;
use typings::persist::ship::{Fitting, Ship};
use typings::persist::site;

use crate::math::round::advance;
use crate::persist::player::{
    bodge_find_player_in_warp, pop_players_in_warp, read_player_location, write_player_location,
    write_player_ship,
};
use crate::persist::site::{read_site_entries, read_sites, write_site_entries};

mod math;
mod persist;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let app = {
        println!("load static data...");
        let measure = Instant::now();
        let statics = Statics::import_yaml("../typings/static").unwrap();
        println!("  took {:?}", measure.elapsed());

        println!("init persist...");
        let measure = Instant::now();
        persist::ensure_statics(&statics).unwrap();
        println!("  took {:?}", measure.elapsed());

        println!("init webserver...");
        let measure = Instant::now();
        let app = init_webserver();
        println!("  took {:?}", measure.elapsed());
        app
    };

    println!("Starting to listen on http://localhost:8080");
    app.listen("[::]:8080").await?; // Works for both IPv4 and IPv6
    Ok(())
}

fn init_webserver() -> tide::Server<()> {
    let mut app = tide::new();
    app.with(tide::utils::Before(|request: Request<()>| async {
        let method = request.method();
        let path = request.url().path();
        println!("HTTP-REQUEST {} {}", method, path);
        request
    }));
    app.with(After(|mut res: Response| async {
        if let Some(err) = res.error() {
            let msg = format!("Error: {:?}", err);
            eprintln!("HTTP ERROR {}", err);
            // NOTE: You may want to avoid sending error messages in a production server.
            res.set_body(msg);
        }
        Ok(res)
    }));
    app.at("/").get(|_| async {
        Ok(Response::builder(StatusCode::Ok)
            .body("Hello world")
            .content_type(mime::HTML)
            .build())
    });
    app.at("/player/:player/location").get(player_location);
    app.at("/player/:player/ship").get(player_ship);
    app.at("/player/:player/station-assets/:solarsystem/:station")
        .get(station_assets);

    app.at("/sites/:solarsystem").get(sites);
    app.at("/sites/:solarsystem/:unique").get(site_entities);

    app.at("/set-instructions/:player")
        .post(testing_set_instructions);

    app
}

fn tide_json_response<T>(body: &T) -> tide::Result
where
    T: ?Sized + serde::Serialize,
{
    Ok(Response::builder(StatusCode::Ok)
        .body(serde_json::to_string_pretty(body)?)
        .content_type(mime::JSON)
        .build())
}

#[allow(clippy::unused_async)]
async fn player_location(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.to_string();
    let body = if let Ok(location) = read_player_location(&player) {
        location
    } else {
        PlayerLocation::Site(site::Identifier {
            solarsystem: solarsystem::Identifier::default(),
            site_unique: site::Info::generate_station(solarsystem::Identifier::default(), 0)
                .site_unique,
        })
    };
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn player_ship(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.to_string();
    let body = if let Ok(ship) = read_player_ship(&player) {
        ship
    } else {
        Ship {
            fitting: Fitting::default(),
            status: calc_max(
                &Statics::import_yaml("../typings/static")?,
                &Fitting::default(),
            )?,
        }
    };
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn site_entities(req: Request<()>) -> tide::Result {
    let solarsystem = req.param("solarsystem")?.parse()?;
    let unique = req.param("unique")?.to_string();
    let body: Vec<typings::frontread::site_entity::SiteEntity> =
        read_site_entries(solarsystem, &unique)
            .map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?
            .iter()
            .map(|o| o.into())
            .collect();
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn sites(req: Request<()>) -> tide::Result {
    let solarsystem = req.param("solarsystem")?.parse()?;
    let body =
        read_sites(solarsystem).map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?;
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn station_assets(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.to_string();
    let solarsystem = req.param("solarsystem")?.parse()?;
    let station = req.param("station")?.parse()?;
    let body = read_station_assets(&player, solarsystem, station);
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn testing_set_instructions(mut req: Request<()>) -> tide::Result {
    let player = req.param("player")?.to_string();
    let instructions = req.body_json::<Vec<Instruction>>().await?;

    let measure = Instant::now();

    println!(
        "Instructions for player {} ({}): {:?}",
        player,
        instructions.len(),
        instructions
    );

    let statics = Statics::import_yaml("../typings/static")?;
    let location = read_player_location(&player).unwrap_or_else(|_| {
        PlayerLocation::Site(site::Identifier {
            solarsystem: solarsystem::Identifier::default(),
            site_unique: site::Info::generate_station(solarsystem::Identifier::default(), 0)
                .site_unique,
        })
    });
    let mut player_locations = HashMap::new();
    player_locations.insert(player.to_string(), location.clone());
    let solarsystem = location.solarsystem();

    let site_unique = match location {
        PlayerLocation::Site(site) => site.site_unique,
        PlayerLocation::Warp(_) => bodge_find_player_in_warp(solarsystem, &player)?,
        PlayerLocation::Station(_) => site::Info::generate_station(solarsystem, 0).site_unique,
    };

    let mut site_entities = read_site_entries(solarsystem, &site_unique).unwrap_or_default();

    let ship = read_player_ship(&player).unwrap_or_else(|_| Ship {
        fitting: Fitting::default(),
        status: calc_max(&statics, &Fitting::default()).unwrap(),
    });
    let mut player_ships = HashMap::new();
    player_ships.insert(player.to_string(), ship);

    let mut player_instructions = HashMap::new();
    player_instructions.insert(player.to_string(), instructions);

    let players_warping_in = pop_players_in_warp(solarsystem, &site_unique);

    advance(
        &statics,
        &(site::Identifier {
            solarsystem,
            site_unique: site_unique.to_string(),
        }),
        &mut site_entities,
        &player_instructions,
        &mut player_locations,
        &mut player_ships,
        &players_warping_in,
    )?;

    write_site_entries(solarsystem, &site_unique, &site_entities)?;
    for (player, ship) in player_ships {
        write_player_ship(&player, &ship)?;
    }
    for (player, location) in player_locations {
        write_player_location(&player, &location)?;
    }

    println!("  instructions took {:?}", measure.elapsed());
    Ok(Response::builder(StatusCode::Ok).build())
}
