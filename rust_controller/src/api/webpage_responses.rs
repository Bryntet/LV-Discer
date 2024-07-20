use super::super::dto;
use crate::api::guard::CoordinatorLoader;
use crate::api::{mutation, query, Coordinator, GeneralChannel, GroupSelectionUpdate, Error};
use crate::dto::CoordinatorBuilder;
use itertools::Itertools;
use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::State;
use rocket_dyn_templates::Template;
use rocket_okapi::openapi;
use serde_json::json;
use tokio::sync::broadcast::Sender;

#[openapi(tag = "HTMX")]
#[get("/focused-players")]
pub async fn focused_players(coordinator: Coordinator) -> Template {
    let players = query::focused_players(coordinator).await.into_inner();
    Template::render("current_selected", json!({"players": players }))
}

#[openapi(tag = "Config")]
#[post("/group/<group_id>")]
pub async fn set_group(
    coordinator: Coordinator,
    group_id: &str,
    updater: &State<GeneralChannel<GroupSelectionUpdate>>,
) -> Result<Template, &'static str> {
    mutation::set_group(coordinator.clone(), group_id, updater).await?;

    let coordinator = coordinator.lock().await;
    Ok(Template::render(
        "current_selected",
        json!({"players":dto::current_dto_players(&coordinator)}),
    ))
}

#[openapi(tag = "HTMX")]
#[post("/init", data = "<builder>")]
pub async fn load(
    loader: &State<CoordinatorLoader>,
    builder: Form<CoordinatorBuilder>,
) -> Result<Template, Error> {
    let coordinator = builder.into_inner().into_coordinator().await?;
    let mut groups = coordinator.groups().into_iter().cloned().collect_vec();
    groups.reverse();
    *loader.0.lock().await = Some(coordinator.into());
    Ok(Template::render("index", json!({"groups": groups})))
}

#[openapi(tag = "HTMX")]
#[get("/")]
pub async fn index(coordinator: Coordinator) -> RawHtml<Template> {
    let coordinator = coordinator.lock().await;
    let mut groups = coordinator.groups();
    groups.reverse();
    let context = json!({"groups": groups});
    RawHtml(Template::render("index", context))
}
