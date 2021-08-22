use space_game_typings::fixed::Statics;
use space_game_typings::frontrw::station_instruction::StationInstruction;
use space_game_typings::persist::player::Player;
use space_game_typings::persist::player_location::PlayerLocation;
use space_game_typings::site::instruction::Instruction;
use space_game_typings::site::Entity;
use tide::http::mime;
use tide::utils::After;
use tide::{Request, Response, StatusCode};

use crate::persist::player::{
    add_player_site_instructions, list_players_with_site_log, pop_player_site_log,
    read_player_generals, read_player_location, read_player_site_instructions, read_station_assets,
};
use crate::persist::site::{read_entitiy_warping, read_site_entities, read_sites};
use crate::station;

mod site_entity;

pub fn init() -> tide::Server<()> {
    let mut app = tide::new();

    #[cfg(debug_assertions)]
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

    app.at("/player/:player/generals").get(player_generals);
    app.at("/player/:player/location").get(player_location);
    app.at("/player/:player/ship").get(player_ship);
    app.at("/player/:player/station-assets/:solarsystem/:station")
        .get(station_assets);
    app.at("/player/:player/site-instructions")
        .get(get_site_instructions)
        .post(post_site_instructions);
    app.at("/player/:player/site-log").get(get_player_site_log);
    app.at("/player/:player/station-instructions")
        .post(post_station_instructions);

    app.at("/platform/:platform/site-log-players")
        .get(get_platform_site_log_players);

    app.at("/sites/:solarsystem").get(sites);
    app.at("/sites/:solarsystem/:unique").get(site_entities);

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
async fn player_generals(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.parse()?;
    let body = read_player_generals(player);
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn player_location(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.parse()?;
    let body = read_player_location(player);
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn player_ship(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.parse()?;
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
        PlayerLocation::Station(s) => read_station_assets(player, s.solarsystem, s.station)
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

#[allow(clippy::unused_async)]
async fn site_entities(req: Request<()>) -> tide::Result {
    let solarsystem = req.param("solarsystem")?.parse()?;
    let site = req.param("unique")?.parse()?;
    let body = site_entity::read(&Statics::default(), solarsystem, site);
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
    let player = req.param("player")?.parse()?;
    let solarsystem = req.param("solarsystem")?.parse()?;
    let station = req.param("station")?.parse()?;
    let body = read_station_assets(player, solarsystem, station);
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn get_site_instructions(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.parse()?;
    let body = read_player_site_instructions(player);
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn post_site_instructions(mut req: Request<()>) -> tide::Result {
    let player = req.param("player")?.parse()?;
    let instructions = req.body_json::<Vec<Instruction>>().await?;
    println!(
        "SiteInstructions for player {:?} ({}): {:?}",
        player,
        instructions.len(),
        instructions
    );
    add_player_site_instructions(player, &instructions)?;
    Ok(Response::builder(StatusCode::Ok).build())
}

#[allow(clippy::unused_async)]
async fn get_player_site_log(req: Request<()>) -> tide::Result {
    let player = req.param("player")?.parse()?;
    let body = pop_player_site_log(player)?;
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn get_platform_site_log_players(req: Request<()>) -> tide::Result {
    let platform = req.param("platform")?;
    let site_log_players = list_players_with_site_log();
    let body = match platform {
        "telegram" => site_log_players
            .iter()
            .filter(|o| matches!(o, Player::Telegram(_))),
        _ => {
            return Err(tide::Error::from_str(
                StatusCode::BadRequest,
                "platform unknown",
            ));
        }
    }
    .collect::<Vec<_>>();
    tide_json_response(&body)
}

#[allow(clippy::unused_async)]
async fn post_station_instructions(mut req: Request<()>) -> tide::Result {
    let player = req.param("player")?.parse()?;
    let instructions = req.body_json::<Vec<StationInstruction>>().await?;
    println!(
        "StationInstructions for player {:?} ({}): {:?}",
        player,
        instructions.len(),
        instructions
    );
    let statics = Statics::default();
    station::do_instructions(&statics, player, &instructions)?;
    Ok(Response::builder(StatusCode::Ok).build())
}
