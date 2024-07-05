use rocket::serde::json::Json;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use serde::Deserialize;

use crate::api::MyError;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};

#[derive(Default, Deserialize, JsonSchema, FromForm)]
pub struct CoordinatorBuilder {
    ip: String,
    event_id: String,
}


impl CoordinatorBuilder {
    pub fn new(ip: String, event_id: String) -> Self {
        Self { ip, event_id }
    }
}

impl CoordinatorBuilder {
    pub async fn into_coordinator(self) -> Result<FlipUpVMixCoordinator, MyError> {
        FlipUpVMixCoordinator::new(self.ip, self.event_id, 0).await
    }
}

impl From<Json<CoordinatorBuilder>> for CoordinatorBuilder {
    fn from(json: Json<CoordinatorBuilder>) -> Self {
        json.into_inner()
    }
}
