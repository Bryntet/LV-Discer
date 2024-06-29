mod mutation;
mod query;
mod vmix_calls;
mod coordinator_wrapper;
mod guard;


use std::sync::Arc;
use query::*;
use rocket::{Build, Request, Rocket};
use rocket::http::Status;
use rocket::response::status;
use rocket_okapi::{JsonSchema, openapi_get_routes};
use rocket_okapi::okapi::schemars;
use crate::controller::coordinator::{FlipUpVMixCoordinator};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::rapidoc::{GeneralConfig, make_rapidoc, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use tokio::sync::{Mutex, MutexGuard};
use crate::api::mutation::*;
use crate::api::vmix_calls::*;
use guard::*;
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
    async fn replace(&self, new: FlipUpVMixCoordinator) {
        *self.lock().await = new;
    }
}


pub fn launch() -> Rocket<Build> {


    rocket::build()
        .manage(MyTestWrapper(Mutex::new(None)))
        .mount("/", openapi_get_routes![current_hole,amount_of_rounds,current_round,play_animation,clear_all, rounds_structure,set_focus,load,init,test,set,])
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
