use std::time::Instant;

use tide::http::mime;
use tide::utils::After;
use tide::{Request, Response, StatusCode};
use typings::dynamic::player_location::{PlayerInSite, PlayerLocation};
use typings::dynamic::ship::Fitting;
use typings::dynamic::site;
use typings::dynamic::site_entity::{self, SiteEntity};
use typings::fixed::facility;
use typings::fixed::site::Kind;

mod statics;

const LISTENER: &str = "[::]:8080"; // Works for both IPv4 and IPv6

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    println!("load static data...");
    let bla = Instant::now();
    let statics = statics::Statics::import("../typings/static").unwrap();
    let took = bla.elapsed();
    println!("took {:?}", took);
    println!("statics {:?}", statics);

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
    app.at("/site-inners/:solarsystem/:unique").get(site_inners);
    app.at("/sites/:solarsystem").get(sites);

    println!("http://localhost:8080");
    app.listen(LISTENER).await?;
    Ok(())
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

    let result = PlayerLocation::Site(PlayerInSite {
        solarsystem: "system1".into(),
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
async fn site_inners(req: Request<()>) -> tide::Result<Response> {
    let solarsystem = req.param("solarsystem")?.to_string();
    let unique = req.param("unique")?.to_string();
    println!("site_inners args: {} {}", solarsystem, unique);

    let result = site::Inners {
        entities: vec![
            SiteEntity::Facility(site_entity::Facility {
                id: facility::Identifier::Station,
            }),
            SiteEntity::Npc(site_entity::Npc {
                shiplayout: "shiplayoutRookieShip".into(),
            }),
            SiteEntity::Lifeless(site_entity::Lifeless {
                id: "lifelessAsteroid".into(),
            }),
            SiteEntity::Lifeless(site_entity::Lifeless {
                id: "lifelessAsteroid".into(),
            }),
            SiteEntity::Npc(site_entity::Npc {
                shiplayout: "shiplayoutFrigate".into(),
            }),
            SiteEntity::Player(site_entity::Player {
                id: "player-dummy-0".into(),
                shiplayout: "shiplayoutRookieShip".into(),
            }),
        ],
    };

    let body = serde_json::to_string_pretty(&result)?;
    Ok(Response::builder(StatusCode::Ok)
        .body(body)
        .content_type(mime::JSON)
        .build())
}

#[allow(clippy::unused_async)]
async fn sites(req: Request<()>) -> tide::Result<Response> {
    let solarsystem = req.param("solarsystem")?.to_string();
    println!("sites args: {}", solarsystem);

    let mut result = site::SitesNearPlanet::new();

    result.insert(
        1,
        vec![site::Info {
            kind: Kind::AsteroidField,
            unique: "backend".into(),
            name: None,
        }],
    );
    result.insert(
        2,
        vec![
            site::Info {
                kind: Kind::FacilityStation,
                unique: "station1".into(),
                name: Some("Wabinihwa I".into()),
            },
            site::Info {
                kind: Kind::AsteroidField,
                unique: "isnt".into(),
                name: None,
            },
            site::Info {
                kind: Kind::AsteroidField,
                unique: "creative".into(),
                name: None,
            },
        ],
    );
    result.insert(
        3,
        vec![site::Info {
            kind: Kind::FacilityStargate,
            unique: "system2".into(),
            name: Some("Liagi".into()),
        }],
    );
    result.insert(
        4,
        vec![
            site::Info {
                kind: Kind::FacilityStargate,
                unique: "system4".into(),
                name: Some("Arama".into()),
            },
            site::Info {
                kind: Kind::AsteroidField,
                unique: "yet".into(),
                name: None,
            },
        ],
    );

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

fn default_status() -> typings::dynamic::ship::Status {
    typings::dynamic::ship::Status {
        capacitor: 40,
        hitpoints_armor: 20,
        hitpoints_structure: 10,
    }
}
