use crate::api::guard::CoordinatorLoader;
use crate::api::{Coordinator, SelectionUpdate};
use crate::dto::CoordinatorBuilder;
use rocket::State;
use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket_dyn_templates::Template;
use rocket_okapi::openapi;
use serde_json::json;
use crate::dto;
use rocket::tokio::sync::broadcast::Sender;

#[openapi(tag = "Config")]
#[post("/focused-player/<focused_player>")]
pub async fn set_focus(focused_player: usize, coordinator: Coordinator, updater: &State<Sender<SelectionUpdate>>) {
    coordinator.lock().await.set_focused_player(focused_player, Some(updater));
}

#[openapi(tag = "Config")]
#[post("/init", data = "<builder>")]
pub async fn load(loader: &State<CoordinatorLoader>, builder: Form<CoordinatorBuilder>) {
    let coordinator = builder.into_inner().into_coordinator().await.unwrap();
    debug!("{:#?}", &coordinator.focused_player());
    *loader.0.lock().await = Some(coordinator.into());
}

#[openapi(tag = "Config")]
#[post("/round/<round_number>")]
pub async fn set_round(coordinator: Coordinator, round_number: usize) {
    let mut coordinator = coordinator
        .lock()
        .await;
    coordinator
        .set_round(round_number);
}

#[openapi(tag = "Config")]
#[post("/group/<group_id>")]
pub async fn set_group(coordinator: Coordinator, group_id: &str, updater: &State<Sender<SelectionUpdate>>) -> Result<Template, &'static str> {
    let mut coordinator = coordinator
        .lock()
        .await;
    coordinator
        .set_group(group_id, Some(updater))
        .ok_or("Unable to set group")?;
    
    
    
    Ok(Template::render("current_selected", json!({"players":dto::current_dto_players(&coordinator)})))
}
#[catch(424)]
pub fn make_coordinator() -> RawHtml<Template> {
    RawHtml(Template::render("new_coordinator", json!({})))
}
