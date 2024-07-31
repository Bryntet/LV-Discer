use std::ops::Deref;

use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::State;
use rocket_dyn_templates::Template;
use rocket_okapi::openapi;
use serde_json::json;

use crate::api::guard::CoordinatorLoader;
use crate::api::websocket::channels::DivisionUpdate;
use crate::api::websocket::htmx::division_updater;
use crate::api::{Coordinator, Error, GeneralChannel, PlayerManagerUpdate};
use crate::dto;
use crate::dto::{CoordinatorBuilder, HoleSetting};

#[openapi(tag = "Config")]
#[post("/player/focused/set/<focused_player>")]
pub async fn set_focus(
    focused_player: usize,
    coordinator: Coordinator,
    player_updater: &GeneralChannel<PlayerManagerUpdate>,
    division_updater: &GeneralChannel<DivisionUpdate>,
) -> Result<Json<dto::Player>, Error> {
    let mut coordinator = coordinator.lock().await;

    coordinator.set_focused_player(focused_player, player_updater, division_updater)?;

    let player = coordinator.focused_player().clone();

    Ok(dto::Player::from_normal_player(player, None).into())
}

#[openapi(tag = "Config")]
#[post("/init", data = "<builder>")]
pub async fn load(loader: &State<CoordinatorLoader>, builder: Json<CoordinatorBuilder>) {
    let coordinator = builder.into_inner().into_coordinator().await.unwrap();
    *loader.0.lock().await = Some(coordinator.into());
}

#[openapi(tag = "Config")]
#[post("/group/<group_id>")]
pub async fn set_group(
    coordinator: Coordinator,
    group_id: &str,
    updater: &GeneralChannel<PlayerManagerUpdate>,
    division_updater: &GeneralChannel<DivisionUpdate>,
) -> Result<(), Error> {
    let mut coordinator = coordinator.lock().await;
    coordinator.set_group(group_id, updater)?;
    coordinator.leaderboard_division = coordinator.focused_player().division.clone();
    division_updater.send(coordinator.deref());
    Ok(())
}

#[openapi(tag = "Live Update")]
#[post("/players/focused/set-group")]
pub async fn set_group_to_focused_player(
    coordinator: Coordinator,
    updater: &GeneralChannel<PlayerManagerUpdate>,
    division_updater: &GeneralChannel<DivisionUpdate>,
) -> Result<(), Error> {
    let mut coordinator = coordinator.lock().await;
    coordinator.update_group_to_focused_player_group(updater)?;
    coordinator.leaderboard_division = coordinator.focused_player().division.clone();
    division_updater.send(coordinator.deref());
    Ok(())
}

#[openapi(tag = "Live Update")]
#[post("/player/<player_id>/throw/set/<throws>")]
pub async fn set_throws(
    coordinator: Coordinator,
    player_id: &str,
    throws: u8,
) -> Result<(), Error> {
    let player_id = player_id.to_string();
    let mut coordinator = coordinator.lock().await;
    let player = coordinator
        .find_player_mut(&player_id)
        .ok_or(Error::PlayerNotFound(player_id))?;
    player.throws = throws;
    std::mem::drop(coordinator);
    Ok(())
}

#[openapi(tag = "Live Update")]
#[post("/player/<player_id>/score/ready")]
pub async fn set_score_ready(coordinator: Coordinator, player_id: &str) -> Result<(), Error> {
    let mut coordinator = coordinator.lock().await;
    coordinator
        .find_player_mut(player_id)
        .ok_or(Error::PlayerNotFound(player_id.to_string()))?
        .results
        .current_result_mut(1);
    Ok(())
}

#[openapi(tag = "Queue")]
#[post("/player/<player_id>/add-to-queue", data = "<hole_setting>")]
pub async fn add_to_queue(
    coordinator: Coordinator,
    channel: &GeneralChannel<PlayerManagerUpdate>,
    player_id: &str,
    hole_setting: Form<dto::HoleSetting>,
) -> Result<(), Error> {
    let (hole, throws) = dbg!((hole_setting.hole, hole_setting.throws));
    coordinator
        .lock()
        .await
        .add_to_queue(player_id.to_string(), hole, throws, channel);
    Ok(())
}

#[openapi(tag = "Queue")]
#[post("/players/queue/next")]
pub async fn next_queue(
    coordinator: Coordinator,
    channel: &GeneralChannel<PlayerManagerUpdate>,
) -> Result<(), Error> {
    coordinator.lock().await.next_queued(channel)?;
    Ok(())
}

#[openapi(tag = "Division")]
#[post("/div/<division>/set")]
pub async fn update_division(
    co: Coordinator,
    division: &str,
    channel: &GeneralChannel<DivisionUpdate>,
) -> Result<(), Error> {
    let mut co = co.lock().await;
    let div = co
        .all_divs
        .iter()
        .find(|div| div.name == division)
        .ok_or(Error::InvalidDivision(division.to_string()))?
        .to_owned();
    co.set_div(&div, channel);
    Ok(())
}

#[catch(424)]
pub fn make_coordinator() -> RawHtml<Template> {
    RawHtml(Template::render("new_coordinator", json!({})))
}
