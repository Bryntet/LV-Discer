use crate::api::{Coordinator, MyError};
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::dto;
use crate::dto::CoordinatorBuilder;
use rocket::serde::json::Json;
use rocket::{tokio, State};
use rocket_okapi::openapi;
use std::sync::Mutex;
use tokio::task::spawn_blocking;
use crate::api::guard::CoordinatorLoader;

#[openapi(tag = "Config")]
#[post("/focused-player/<focused_player>")]
pub async fn set_focus(focused_player: &str, coordinator: Coordinator) {
    //coordinator.lock().await.set_player(focused_player)
    todo!()
}



#[openapi(tag = "Config")]
#[post("/init", data = "<builder>")]
pub async fn load(loader: &State<CoordinatorLoader>, builder: Json<CoordinatorBuilder>)  {
    let coordinator = builder.into_inner().into_coordinator().await.unwrap();
    debug!("{:#?}",&coordinator.focused_player());
    *loader.0.lock().await = Some(coordinator.into());
    
}
