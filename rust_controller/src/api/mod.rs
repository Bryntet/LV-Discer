mod mutation;
mod query;
mod vmix_calls;
mod coordinator_wrapper;

use query::*;
use rocket::{Build, Rocket};
use rocket_okapi::{openapi_get_routes};

use crate::controller::coordinator::{FlipUpVMixCoordinator};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::rapidoc::{GeneralConfig, make_rapidoc, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use crate::api::vmix_calls::*;


pub fn launch() -> Rocket<Build> {
    let coordinator = FlipUpVMixCoordinator::default();
    rocket::build()
        .manage(coordinator)
        .mount("/", openapi_get_routes![current_hole,amount_of_rounds,current_round,play_animation,clear_all])
        .mount("/swagger", make_swagger_ui(&SwaggerUIConfig{
            url: "../openapi.json".to_owned(),
            ..Default::default()
        }))
        .mount("/", make_rapidoc(&RapiDocConfig{
            general: GeneralConfig {
                spec_urls: vec![UrlObject::new("General", "./openapi.json")],
                ..Default::default()
            },
            ..Default::default()
        }))
}
