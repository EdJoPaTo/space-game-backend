use std::time::Instant;

use persist::player::read_station_assets;
use tide::http::mime;
use tide::utils::After;
use tide::{Request, Response, StatusCode};
use typings::fixed::site::Kind;
use typings::persist::player_location::{self, PlayerLocation};
use typings::persist::ship::Fitting;
use typings::persist::site;

use crate::persist::player::read_player_location;
use crate::persist::site::{read_site_entries, read_sites};

mod persist;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let app = {
        println!("load static data...");
        let measure = Instant::now();
        let statics = typings::fixed::Statics::import_yaml("../typings/static").unwrap();
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
        let mime = request.content_type();
        let method = request.method();
        let path = request.url().path();
        println!("incoming {} {:?} {}", method, mime, path);
        request
    }));
    app.with(After(|mut res: Response| async {
        if let Some(err) = res.error() {
            let msg = format!("Error: {:?}", err);
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
    app.at("/player-location/:player").get(player_location);
    app.at("/sites/:solarsystem").get(sites);
    app.at("/sites/:solarsystem/:unique").get(site_entities);
    app.at("/station-assets/:player/:solarsystem/:station")
        .get(station_assets);
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
    println!("player_location: {}", player);
    let body = if let Ok(location) = read_player_location(&player) {
        location
    } else {
        let site = site::Info {
            kind: Kind::FacilityStation,
            unique: "station1".into(),
            name: Some("Wabinihwa I".into()),
        };
        PlayerLocation::Site(player_location::Site {
            solarsystem: "Wabinihwa".into(),
            site,
            ship_fitting: Fitting::default(),
            ship_status: default_status(),
        })
    };
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn site_entities(req: Request<()>) -> tide::Result {
    let solarsystem = req.param("solarsystem")?.to_string();
    let unique = req.param("unique")?.to_string();
    let body: Vec<typings::frontread::site_entity::SiteEntity> =
        read_site_entries(&solarsystem, &unique)
            .map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?
            .iter()
            .map(|o| o.into())
            .collect();
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn sites(req: Request<()>) -> tide::Result {
    let solarsystem = req.param("solarsystem")?.to_string();
    let body =
        read_sites(&solarsystem).map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?;
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn station_assets(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.to_string();
    let solarsystem = req.param("solarsystem")?.to_string();
    let station = req.param("station")?.parse()?;
    let body = read_station_assets(&player, &solarsystem, station).unwrap_or_default();
    tide_json_response(&body)
}

fn default_status() -> typings::persist::ship::Status {
    typings::persist::ship::Status {
        capacitor: 40,
        hitpoints_armor: 20,
        hitpoints_structure: 10,
    }
}
