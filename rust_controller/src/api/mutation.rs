use crate::api::Coordinator;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::dto;
use crate::dto::CoordinatorBuilder;
use rocket::serde::json::Json;
use rocket::{tokio, State};
use rocket_okapi::openapi;
use std::sync::Mutex;
use tokio::task::spawn_blocking;

#[openapi(tag = "Config")]
#[post("/focused-player/<focused_player>")]
pub async fn set_focus(focused_player: &str, coordinator: Coordinator) {
    //coordinator.lock().await.set_player(focused_player)
    ()
}

#[openapi(tag = "Config")]
#[post("/load")]
pub async fn load(coordinator: Coordinator) {
    let mut c = coordinator.lock().await;
    //c.fetch_event().await;

    //dbg!(&c);
    //dbg!(c.focused_player());
}

#[openapi(tag = "Config")]
#[post("/init", data = "<builder>")]
pub async fn init(builder: Json<CoordinatorBuilder>, coordinator: Coordinator) {
    coordinator.replace(builder.into_inner().into()).await
}
