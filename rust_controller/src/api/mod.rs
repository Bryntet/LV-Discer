mod coordinator_wrapper;
mod guard;
mod mutation;
mod query;
mod vmix_calls;
mod websocket;
mod webpage_responses;

use mutation::*;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use guard::*;

use rocket::{Build, Rocket, Route};
use rocket_dyn_templates::Template;
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::openapi_get_routes;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
pub use websocket::SelectionUpdate;
use rocket::tokio::sync::broadcast::channel;

pub use guard::MyError;

#[derive(Debug, Clone)]
struct Coordinator(Arc<Mutex<FlipUpVMixCoordinator>>);

impl From<FlipUpVMixCoordinator> for Coordinator {
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
    use query::*;
    use mutation::*;
    use vmix_calls::*;
    use webpage_responses::{index,okapi_add_operation_for_index_};
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
        index
        
    ]
}

fn get_websocket_routes() -> Vec<Route> {
    use websocket::*;
    routes![selection_updater,focused_player_changer]
}
fn get_webpage_routes() -> Vec<Route> {
    use webpage_responses::*;
    
    openapi_get_routes![focused_players,set_group,load]
}

pub fn launch() -> Rocket<Build> {
    
    let (sender, _) = channel::<websocket::SelectionUpdate>(1024);
    
    rocket::build()
        .configure(rocket::Config {
            address: IpAddr::V4("10.169.122.114".parse().unwrap()),
            ..Default::default()
        })
        .manage(CoordinatorLoader(Mutex::new(None)))
        .manage(sender)
        .mount(
            "/",
            get_normal_routes(),
        )
        .mount("/", get_websocket_routes())
        .mount("/htmx/", get_webpage_routes())
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
