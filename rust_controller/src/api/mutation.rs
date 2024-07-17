use crate::api::guard::CoordinatorLoader;
use crate::api::{Coordinator, Error, GeneralChannel, GroupSelectionUpdate};
use crate::dto;
use crate::dto::CoordinatorBuilder;
use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::tokio::sync::broadcast::Sender;
use rocket::State;
use rocket_dyn_templates::Template;
use rocket_okapi::openapi;
use serde_json::json;

#[openapi(tag = "Config")]
#[post("/player/focused/set/<focused_player>")]
pub async fn set_focus(
    focused_player: usize,
    coordinator: Coordinator,
    updater: &GeneralChannel<GroupSelectionUpdate>,
) -> Result<Json<dto::Player>, Error> {
    let mut coordinator = coordinator.lock().await;

    coordinator.set_focused_player(focused_player, Some(updater))?;

    let player = coordinator.focused_player().clone();

    Ok(dto::Player::from_normal_player(player).into())
}

#[openapi(tag = "Config")]
#[post("/init", data = "<builder>")]
pub async fn load(loader: &State<CoordinatorLoader>, builder: Json<CoordinatorBuilder>) {
    let coordinator = builder.into_inner().into_coordinator().await.unwrap();
    *loader.0.lock().await = Some(coordinator.into());
}

#[openapi(tag = "Config")]
#[post("/round/<round_number>")]
pub async fn set_round(coordinator: Coordinator, round_number: usize) {
    let mut coordinator = coordinator.lock().await;
    coordinator.set_round(round_number);
}

#[openapi(tag = "Config")]
#[post("/group/<group_id>")]
pub async fn set_group(
    coordinator: Coordinator,
    group_id: &str,
    updater: &State<GeneralChannel<GroupSelectionUpdate>>,
) -> Result<(), &'static str> {
    let mut coordinator = coordinator.lock().await;
    coordinator
        .set_group(group_id, Some(updater))
        .ok_or("Unable to set group")
}

//#[openapi(tag = "Live Update")]
#[post("/player/<player_id>/throw/set/<throws>")]
pub async fn set_throws(
    coordinator: Coordinator,
    player_id: &str,
    throws: u8,
) -> Result<(), Error> {
    let player_id = player_id.to_string();
    let mut coordinator = coordinator.lock().await;
    let player = coordinator
        .find_player_mut(player_id.clone())
        .ok_or(Error::PlayerNotFound(player_id))?;
    player.throws = throws;
    std::mem::drop(player);
    std::mem::drop(coordinator);
    Ok(())
}

#[openapi(tag = "Live Update")]
#[post("/player/<player_id>/score/ready")]
pub async fn set_score_ready(coordinator: Coordinator, player_id: String) -> Result<(), Error> {
    let mut coordinator = coordinator.lock().await;
    coordinator
        .find_player_mut(player_id.to_string())
        .ok_or(Error::PlayerNotFound(player_id.to_string()))?
        .results
        .current_result_mut(1);
    Ok(())
}

#[catch(424)]
pub fn make_coordinator() -> RawHtml<Template> {
    RawHtml(Template::render("new_coordinator", json!({})))
}
