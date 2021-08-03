use std::time::Instant;

use tide::http::mime;
use tide::utils::After;
use tide::{Request, Response, StatusCode};
use typings::fixed::site::Kind;
use typings::persist::player_location::{self, PlayerLocation};
use typings::persist::ship::Fitting;
use typings::persist::site;

use crate::persist::site::{read_site_entries, read_sites};

mod persist;
mod statics;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let app = {
        println!("load static data...");
        let measure = Instant::now();
        let statics = statics::Statics::import("../typings/static").unwrap();
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
    app.at("/player-location/:playerid").get(player_location);
    app.at("/sites/:solarsystem").get(sites);
    app.at("/sites/:solarsystem/:unique").get(site_entities);
    app
}

#[allow(clippy::unused_async)]
async fn player_location(req: Request<()>) -> tide::Result<Response> {
    let playerid = req.param("playerid")?.to_string();
    println!("player_location: {}", playerid);

    let site = site::Info {
        kind: Kind::FacilityStation,
        unique: "station1".into(),
        name: Some("Wabinihwa I".into()),
    };

    let result = PlayerLocation::Site(player_location::Site {
        solarsystem: "Wabinihwa".into(),
        site,
        ship_fitting: default_fitting(),
        ship_status: default_status(),
    });

    let body = serde_json::to_string_pretty(&result)?;
    Ok(Response::builder(StatusCode::Ok)
        .body(body)
        .content_type(mime::JSON)
        .build())
}

#[allow(clippy::unused_async)]
async fn site_entities(req: Request<()>) -> tide::Result<Response> {
    let solarsystem = req.param("solarsystem")?.to_string();
    let unique = req.param("unique")?.to_string();
    let result: Vec<typings::frontread::site_entity::SiteEntity> =
        read_site_entries(&solarsystem, &unique)
            .map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?
            .iter()
            .map(|o| o.into())
            .collect();
    let body = serde_json::to_string_pretty(&result)?;
    Ok(Response::builder(StatusCode::Ok)
        .body(body)
        .content_type(mime::JSON)
        .build())
}

#[allow(clippy::unused_async)]
async fn sites(req: Request<()>) -> tide::Result<Response> {
    let solarsystem = req.param("solarsystem")?.to_string();
    let result =
        read_sites(&solarsystem).map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?;
    let body = serde_json::to_string_pretty(&result)?;
    Ok(Response::builder(StatusCode::Ok)
        .body(body)
        .content_type(mime::JSON)
        .build())
}

fn default_fitting() -> Fitting {
    Fitting {
        layout: "shiplayoutRookieShip".into(),
        slots_targeted: vec!["modtRookieMiningLaser".into(), "modtRookieLaser".into()],
        slots_untargeted: vec!["moduRookieArmorRepair".into()],
        slots_passive: vec!["modpRookieArmorPlate".into()],
    }
}

fn default_status() -> typings::persist::ship::Status {
    typings::persist::ship::Status {
        capacitor: 40,
        hitpoints_armor: 20,
        hitpoints_structure: 10,
    }
}
