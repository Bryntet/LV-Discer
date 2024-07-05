use rocket::response::content::RawHtml;
use rocket_dyn_templates::Template;
use rocket_okapi::openapi;
use serde_json::json;
use crate::api::{Coordinator, query};


#[openapi(tag = "HTMX")]
#[get("/focused-players")]
pub async fn focused_players(coordinator: Coordinator) -> Template {
    let players = query::focused_players(coordinator).await.into_inner();
    Template::render("current_selected",json!({"players": players }))
}


#[get("/")]
pub async fn index(coordinator: Coordinator) -> RawHtml<Template> {
    let coordinator = coordinator.lock().await;
    let mut groups = coordinator.groups();
    groups.reverse();
    let context = json!({"groups": groups});
    RawHtml(Template::render("index", context))
}