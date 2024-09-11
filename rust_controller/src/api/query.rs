use crate::api::Coordinator;
use crate::dto;

use itertools::Itertools;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

/// # GET current hole
#[openapi(tag = "Hole")]
#[get("/hole/current")]
pub async fn current_hole(coordinator: Coordinator) -> Json<usize> {
    coordinator.lock().await.current_hole().into()
}

/// # GET Amount of rounds
#[openapi(tag = "Round")]
#[get("/rounds")]
pub async fn amount_of_rounds(coordinator: Coordinator) -> Json<usize> {
    coordinator.lock().await.amount_of_rounds().into()
}

/// # GET Current round
#[openapi(tag = "Round")]
#[get("/round")]
pub async fn current_round(coordinator: Coordinator) -> Json<usize> {
    coordinator.lock().await.get_round().into()
}

#[openapi(tag = "Preprocessing")]
#[get("/players")]
pub async fn get_players(coordinator: Coordinator) -> Json<Vec<dto::Player>> {
    coordinator
        .lock()
        .await
        .available_players()
        .into_iter()
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
    coordinator
        .lock()
        .await
        .groups()
        .into_iter()
        .cloned()
        .collect_vec()
        .into()
}

#[openapi(tag = "Player")]
#[get("/players/focused")]
pub async fn focused_players(coordinator: Coordinator) -> Json<Vec<dto::Player>> {
    coordinator.lock().await.dto_players().into()
}

#[openapi(tag = "Player")]
#[get("/player/focused")]
pub async fn focused_player(coordinator: Coordinator) -> Json<dto::Player> {
    let coordinator = coordinator.lock().await;
    dto::Player::from(coordinator.focused_player()).into()
}

#[openapi(tag = "Player")]
#[get("/players/card")]
pub async fn focused_card(coordinator: Coordinator) -> Json<Vec<dto::Player>> {
    let coordinator = coordinator.lock().await;
    coordinator.dto_card().into()
}
