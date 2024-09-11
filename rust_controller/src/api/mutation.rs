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
use crate::api::websocket::{hole_finished_alert, HoleFinishedAlert, LeaderboardRoundUpdate};
use crate::api::{Coordinator, Error, GeneralChannel, PlayerManagerUpdate};
use crate::dto;
use crate::dto::{CoordinatorBuilder, HoleSetting};

#[openapi(tag = "Config")]
#[post("/player/focused/set/<focused_player>")]
pub async fn set_focus(
    focused_player: usize,
    coordinator: Coordinator,
    player_updater: GeneralChannel<PlayerManagerUpdate>,
    division_updater: GeneralChannel<DivisionUpdate>,
) -> Result<Json<dto::Player>, Error> {
    let mut coordinator = coordinator.lock().await;

    coordinator.set_focused_player(focused_player, player_updater, division_updater)?;

    let player = coordinator.focused_player().clone();

    Ok(dto::Player::from_normal_player(player, None).into())
}

#[openapi(tag = "Config")]
#[post("/init", data = "<builder>")]
pub async fn load(
    loader: &State<CoordinatorLoader>,
    builder: Json<CoordinatorBuilder>,
    hole_finished_alert: GeneralChannel<HoleFinishedAlert>,
) {
    let coordinator = builder.into_inner().into_coordinator().await.unwrap();
    *loader.0.lock().await = Some(coordinator.into_coordinator(hole_finished_alert).await);
}

#[openapi(tag = "Config")]
#[post("/group/<group_id>")]
pub async fn set_group(
    coordinator: Coordinator,
    group_id: &str,
    updater: GeneralChannel<PlayerManagerUpdate>,
    division_updater: GeneralChannel<DivisionUpdate>,
) -> Result<(), Error> {
    let mut coordinator = coordinator.lock().await;
    coordinator.set_group(group_id, updater)?;
    coordinator.leaderboard_division = coordinator.focused_player().division.clone();

    division_updater.send_from_coordinator(coordinator.deref());
    Ok(())
}

#[openapi(tag = "Leaderboard")]
#[post("/leaderboard/next-10")]
pub async fn next_10_lb(coordinator: Coordinator) {
    coordinator.lock().await.increase_leaderboard_skip();
}
#[openapi(tag = "Leaderboard")]
#[post("/leaderboard/reset-pos")]
pub async fn reset_lb_pos(coordinator: Coordinator) {
    coordinator.lock().await.reset_leaderboard_skip();
}

#[openapi(tag = "Leaderboard")]
#[post("/leaderboard/rewind-pos")]
pub async fn rewind_lb_pos(coordinator: Coordinator) {
    coordinator.lock().await.decrease_leaderboard_skip();
}

#[openapi(tag = "Live Update")]
#[post("/players/focused/set-group")]
pub async fn set_group_to_focused_player(
    coordinator: Coordinator,
    updater: GeneralChannel<PlayerManagerUpdate>,
    division_updater: GeneralChannel<DivisionUpdate>,
) -> Result<(), Error> {
    let mut coordinator = coordinator.lock().await;
    coordinator.update_group_to_focused_player_group(updater)?;
    coordinator.leaderboard_division = coordinator.focused_player().division.clone();
    division_updater.send_from_coordinator(coordinator.deref());
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
    channel: GeneralChannel<PlayerManagerUpdate>,
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
    channel: GeneralChannel<PlayerManagerUpdate>,
) -> Result<(), Error> {
    coordinator.lock().await.next_queued(channel)?;
    Ok(())
}

#[openapi(tag = "Division")]
#[post("/div/<division>/set")]
pub async fn update_division(
    co: Coordinator,
    division: &str,
    channel: GeneralChannel<DivisionUpdate>,
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

#[openapi(tag = "Division")]
#[post("/div/set?<division>")]
pub async fn update_division_form(
    co: Coordinator,
    division: &str,
    channel: GeneralChannel<DivisionUpdate>,
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

#[openapi(tag = "Featured hole")]
#[post("/featured-hole/update-card")]
pub async fn update_featured_hole_group(co: Coordinator) {
    co.lock().await.update_featured_card();
}

#[openapi(tag = "Featured hole")]
#[post("/featured-hole/next-card")]
pub async fn next_featured_hole_card(coordinator: Coordinator) {
    coordinator.lock().await.next_featured_card();
}

#[openapi(tag = "Featured hole")]
#[post("/featured-hole/rewind-card")]
pub async fn rewind_featured_hole_card(coordinator: Coordinator) {
    coordinator.lock().await.rewind_card();
}

#[openapi(tag = "Leaderboard")]
#[post("/leaderboard/round/<round>")]
pub async fn set_leaderboard_round(
    coordinator: Coordinator,
    round: usize,
    watcher: GeneralChannel<LeaderboardRoundUpdate>,
) {
    let round = round - 1;
    let mut co = coordinator.lock().await;
    co.leaderboard_round = round;
    watcher.send_from_coordinator(&co);
    co.reset_leaderboard_skip();
    co.set_leaderboard(None);
}

#[openapi(tag = "Hole")]
#[post("/set-hole/<hole>")]
pub async fn set_hole(coordinator: Coordinator, hole: usize) {
    let mut co = coordinator.lock().await;
    co.make_hole_info(Some(hole - 1));
}

#[catch(424)]
pub fn make_coordinator() -> RawHtml<Template> {
    RawHtml(Template::render("new_coordinator", json!({})))
}
