use crate::api::guard::CoordinatorLoader;
use crate::api::{Coordinator, MyError};
use crate::dto::CoordinatorBuilder;
use rocket::serde::json::Json;
use rocket::{tokio, State};
use rocket_dyn_templates::Template;
use rocket_okapi::openapi;
use serde_json::{json, Value};

#[openapi(tag = "Config")]
#[post("/focused-player/<focused_player>")]
pub async fn set_focus(focused_player: &str, coordinator: Coordinator) {
    //coordinator.lock().await.set_player(focused_player)
    todo!()
}

#[openapi(tag = "Config")]
#[post("/init", data = "<builder>")]
pub async fn load(loader: &State<CoordinatorLoader>, builder: Json<CoordinatorBuilder>) {
    let coordinator = builder.into_inner().into_coordinator().await.unwrap();
    debug!("{:#?}", &coordinator.focused_player());
    *loader.0.lock().await = Some(coordinator.into());
}

#[openapi(tag = "Config")]
#[post("/group/<group_id>")]
pub async fn set_group(coordinator: Coordinator, group_id: &str) -> Result<Template, &'static str> {
    let mut coordinator = coordinator
        .lock()
        .await;
    coordinator
        .set_group(group_id)
        .ok_or("Unable to set group")?;
    let players= coordinator.current_players().clone();
    Ok(Template::render("current_selected", json!({"players":players})))
}
