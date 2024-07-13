mod coordinator_wrapper;
mod guard;
mod mutation;
mod query;
mod vmix_calls;
mod webpage_responses;
mod websocket;

use crate::controller::coordinator::FlipUpVMixCoordinator;
use guard::*;
use mutation::*;

use rocket::config::LogLevel;
use rocket::tokio::sync::broadcast::channel;
use rocket::{Build, Rocket, Route};
use rocket_dyn_templates::Template;
use rocket_okapi::openapi_get_routes;
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
pub use websocket::channels::{GroupSelectionUpdate, HoleUpdate};

pub use crate::api::websocket::channels::GeneralChannel;
pub use guard::Error;

#[derive(Debug, Clone)]
struct Coordinator(Arc<Mutex<FlipUpVMixCoordinator>>);

impl<'a> From<FlipUpVMixCoordinator> for Coordinator {
    fn from(value: FlipUpVMixCoordinator) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}
impl Coordinator {
    async fn lock(&self) -> MutexGuard<FlipUpVMixCoordinator> {
        self.0.lock().await
    }
}

fn get_normal_routes() -> Vec<Route> {
    use mutation::*;
    use query::*;
    use vmix_calls::*;
    use webpage_responses::{index, okapi_add_operation_for_index_};
    openapi_get_routes![
        current_hole,
        amount_of_rounds,
        current_round,
        rounds_structure,
        get_players,
        get_divisions,
        set_focus,
        load,
        set_group,
        set_round,
        play_animation,
        clear_all,
        get_groups,
        index,
        update_leaderboard, // TODO
        increase_score,
        focused_players,
        focused_player,
        revert_score,
        increase_throw,
        revert_throw,
        play_ob_animation,
        set_hole_info,
        update_other_leaderboard,
    ]
}

fn get_websocket_routes() -> Vec<Route> {
    use websocket::*;
    routes![selection_watcher, hole_watcher]
}

fn get_websocket_htmx_routes() -> Vec<Route> {
    use websocket::htmx::*;
    routes![selection_updater, focused_player_changer]
}

fn get_webpage_routes() -> Vec<Route> {
    use webpage_responses::*;

    openapi_get_routes![focused_players, set_group, load]
}

pub fn launch() -> Rocket<Build> {
    let (group_selection_sender, _) = channel::<websocket::GroupSelectionUpdate>(1024);
    let group_selection_sender = GeneralChannel::from(group_selection_sender);
    let (hole_update_sender, _) = channel::<HoleUpdate>(1024);
    let hole_update_sender = GeneralChannel::from(hole_update_sender);

    rocket::build()
        .configure(rocket::Config {
            address: IpAddr::V4("10.169.122.114".parse().unwrap()),
            ..Default::default()
        })
        .manage(CoordinatorLoader(Mutex::new(None)))
        .manage(group_selection_sender)
        .manage(hole_update_sender)
        .mount("/", get_normal_routes())
        .mount("/htmx/", get_webpage_routes())
        .mount("/ws", get_websocket_routes())
        .mount("/ws/htmx/", get_websocket_htmx_routes())
        .attach(Template::fairing())
        .register("/", catchers![make_coordinator,])
        .mount(
            "/api/swagger",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/api",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
}
