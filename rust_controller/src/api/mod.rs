mod coordinator_wrapper;
mod guard;
mod mutation;
mod query;
mod vmix_calls;

use crate::api::mutation::*;
use crate::api::vmix_calls::*;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use guard::*;
use query::*;
use rocket::{Build, Rocket};
use rocket_dyn_templates::Template;
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::{openapi_get_routes, JsonSchema};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

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

pub fn launch() -> Rocket<Build> {
    rocket::build()
        .configure(rocket::Config {
            address: IpAddr::V4("10.169.122.114".parse().unwrap()),
            ..Default::default()
        })
        .manage(CoordinatorLoader(Mutex::new(None)))
        .mount(
            "/",
            openapi_get_routes![
                current_hole,
                amount_of_rounds,
                current_round,
                play_animation,
                clear_all,
                rounds_structure,
                set_focus,
                load,
                get_divisions,
                get_groups,
                set_group
            ],
        )
        .mount("/", routes![groups_and_players])
        .attach(Template::fairing())
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
