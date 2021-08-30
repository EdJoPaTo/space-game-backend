#![allow(clippy::unused_async)]

use std::sync::Arc;

use async_std::sync::{Mutex, MutexGuardArc};
use space_game_typings::fixed::Statics;
use space_game_typings::player::location::PlayerLocation;
use space_game_typings::player::Player;
use space_game_typings::site::instruction::Instruction as SiteInstruction;
use space_game_typings::site::Entity;
use space_game_typings::station::instruction::Instruction as StationInstruction;
use tide::http::mime;
use tide::utils::After;
use tide::{Request, Response, StatusCode};

use crate::persist::player::{
    add_player_site_instructions, read_player_location, read_player_site_instructions,
};
use crate::persist::site::{read_entitiy_warping, read_site_entities, read_sites};
use crate::persist::Persist;
use crate::station;

mod site_entity;

#[derive(Clone)]
pub struct State {
    pub statics: Arc<Statics>,
    pub persist: Arc<Mutex<Persist>>,
}

impl State {
    pub async fn persist(&self) -> MutexGuardArc<Persist> {
        self.persist.lock_arc().await
    }
}

pub fn init(state: State) -> tide::Server<State> {
    let mut app = tide::with_state(state);

    #[cfg(debug_assertions)]
    app.with(tide::utils::Before(|request: Request<_>| async {
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

    app.at("/player/:player/generals").get(player_generals);
    app.at("/player/:player/location").get(player_location);
    app.at("/player/:player/ship").get(player_ship);
    app.at("/player/:player/station-assets/:solarsystem/:station")
        .get(station_assets);
    app.at("/player/:player/site-instructions")
        .get(get_site_instructions)
        .post(post_site_instructions);
    app.at("/player/:player/notifications")
        .get(get_player_notifications);
    app.at("/player/:player/station-instructions")
        .post(post_station_instructions);

    app.at("/platform/:platform/notification-players")
        .get(get_platform_players_with_notifications);

    app.at("/sites/:solarsystem").get(sites);
    app.at("/sites/:solarsystem/:unique").get(site_entities);

    app.at("/market/:item").get(get_market);

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

fn tide_parse_param<T>(req: &Request<State>, param: &str) -> tide::Result<T>
where
    T: std::str::FromStr,
{
    let p = req.param(param)?;
    p.parse().map_err(|_| {
        tide::Error::new(
            StatusCode::NotFound,
            anyhow::anyhow!("failed to parse {}", param),
        )
    })
}

async fn player_generals(req: Request<State>) -> tide::Result {
    let player = tide_parse_param(&req, "player")?;
    let body = req.state().persist().await.player_generals.read(player);
    tide_json_response(&body)
}

async fn player_location(req: Request<State>) -> tide::Result {
    let player = tide_parse_param(&req, "player")?;
    let body = read_player_location(player);
    tide_json_response(&body)
}

async fn player_ship(req: Request<State>) -> tide::Result {
    let player = tide_parse_param(&req, "player")?;
    let location = read_player_location(player);
    let body = match location {
        PlayerLocation::Site(s) => {
            let entities = read_site_entities(s.solarsystem, s.site)?;
            let ship = entities
                .iter()
                .find_map(|e| match e {
                    Entity::Player((p, ship)) if p == &player => Some(ship),
                    _ => None,
                })
                .expect("player has to be in the site of its location");
            ship.clone()
        }
        PlayerLocation::Station(s) => req
            .state()
            .persist()
            .await
            .player_station_assets
            .read(player, s.solarsystem, s.station)
            .ships
            .last()
            .cloned()
            .unwrap_or_default(),
        PlayerLocation::Warp(w) => {
            let entities = read_entitiy_warping(w.solarsystem);
            let ship = entities
                .iter()
                .find_map(|(_site, entity)| match entity {
                    Entity::Player((p, ship)) if p == &player => Some(ship),
                    _ => None,
                })
                .expect("player has to be in warp to its location");
            ship.clone()
        }
    };
    tide_json_response(&body)
}

async fn site_entities(req: Request<State>) -> tide::Result {
    let solarsystem = tide_parse_param(&req, "solarsystem")?;
    let site = tide_parse_param(&req, "unique")?;
    let body = site_entity::read(&req.state().statics, solarsystem, site);
    tide_json_response(&body)
}

async fn sites(req: Request<State>) -> tide::Result {
    let solarsystem = tide_parse_param(&req, "solarsystem")?;
    let body =
        read_sites(solarsystem).map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?;
    tide_json_response(&body)
}

async fn station_assets(req: Request<State>) -> tide::Result {
    let player = tide_parse_param(&req, "player")?;
    let solarsystem = tide_parse_param(&req, "solarsystem")?;
    let station = tide_parse_param(&req, "station")?;
    let body = req
        .state()
        .persist()
        .await
        .player_station_assets
        .read(player, solarsystem, station);
    tide_json_response(&body)
}

async fn get_site_instructions(req: Request<State>) -> tide::Result {
    let player = tide_parse_param(&req, "player")?;
    let body = read_player_site_instructions(player);
    tide_json_response(&body)
}

async fn post_site_instructions(mut req: Request<State>) -> tide::Result {
    let player = tide_parse_param(&req, "player")?;
    let instructions = req.body_json::<Vec<SiteInstruction>>().await?;
    println!(
        "SiteInstructions for player {:?} ({}): {:?}",
        player,
        instructions.len(),
        instructions
    );
    add_player_site_instructions(player, &instructions)?;
    Ok(Response::builder(StatusCode::Ok).build())
}

async fn get_player_notifications(req: Request<State>) -> tide::Result {
    let player = tide_parse_param(&req, "player")?;
    let body = req
        .state()
        .persist()
        .await
        .player_notifications
        .pop(player)?;
    tide_json_response(&body)
}

async fn get_platform_players_with_notifications(req: Request<State>) -> tide::Result {
    let platform = req.param("platform")?;
    let site_log_players = req
        .state()
        .persist()
        .await
        .player_notifications
        .list_players();
    let body = match platform {
        "telegram" => site_log_players
            .iter()
            .filter(|o| matches!(o, Player::Telegram(_))),
        _ => {
            return Err(tide::Error::from_str(
                StatusCode::NotFound,
                "platform unknown",
            ));
        }
    }
    .collect::<Vec<_>>();
    tide_json_response(&body)
}

async fn post_station_instructions(mut req: Request<State>) -> tide::Result {
    let player = tide_parse_param(&req, "player")?;
    let instructions = req.body_json::<Vec<StationInstruction>>().await?;
    println!(
        "StationInstructions for player {:?} ({}): {:?}",
        player,
        instructions.len(),
        instructions
    );
    let statics = &req.state().statics;
    let persist = &mut req.state().persist().await;
    station::do_instructions(statics, persist, player, &instructions)?;
    Ok(Response::builder(StatusCode::Ok).build())
}

async fn get_market(req: Request<State>) -> tide::Result {
    let item = tide_parse_param(&req, "item")?;
    let body = req.state().persist().await.market.get(item);
    tide_json_response(&body)
}
