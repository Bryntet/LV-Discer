use crate::api::Coordinator;
use crate::dto;

use itertools::Itertools;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

/// # GET current hole
#[openapi(tag = "Hole")]
#[get("/current-hole")]
pub async fn current_hole(coordinator: Coordinator) -> Json<usize> {
    //coordinator.lock().await.current_hole().into()
    Json(1)
}

/// # GET Amount of rounds
#[openapi(tag = "Round")]
#[get("/rounds")]
pub async fn amount_of_rounds(coordinator: Coordinator) -> Json<usize> {
    //coordinator.lock().await.get_rounds().into()
    Json(1)
}

/// # GET Current round
#[openapi(tag = "Round")]
#[get("/round")]
pub async fn current_round(coordinator: Coordinator) -> Json<usize> {
    //coordinator.lock().await.get_round().into()
    Json(1)
}

/// # Rounds structure
/// Used for preprocessing, i.e. when selecting parameters before coordinator is initialized
#[openapi(tag = "Preprocessing")]
#[get("/event/<event_id>/rounds")]
pub async fn rounds_structure(event_id: String) -> Json<crate::dto::Rounds> {
    crate::dto::get_rounds(event_id).await.unwrap().into()
}

#[openapi(tag = "Preprocessing")]
#[get("/players")]
pub async fn get_players(coordinator: Coordinator) -> Json<Vec<dto::Player>> {
    coordinator
        .lock()
        .await
        .available_players
        .iter()
        .map(dto::Player::from)
        .collect_vec()
        .into()
}

#[openapi(tag = "Preprocessing")]
#[get("/divisions")]
pub async fn get_divisions(coordinator: Coordinator) -> Json<Vec<String>> {
    coordinator.lock().await.get_div_names().into()
}

#[openapi(tag = "Queue System")]
#[get("/groups")]
pub async fn get_groups(coordinator: Coordinator) -> Json<Vec<dto::Group>> {
    coordinator.lock().await.groups().into_iter().cloned().collect_vec().into()
}



#[openapi(tag = "Player")]
#[get("/focused-players")]
pub async fn focused_players(coordinator: Coordinator) -> Json<Vec<dto::Player>> {
    let coordinator = coordinator.lock().await;
    dto::current_dto_players(&coordinator).into()
}
