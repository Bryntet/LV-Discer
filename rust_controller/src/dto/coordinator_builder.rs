use crate::controller::coordinator::FlipUpVMixCoordinator;
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::api::Error;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};

#[derive(Default, Deserialize, JsonSchema, FromForm)]
pub struct CoordinatorBuilder {
    ip: String,
    event_id: String,
    round: usize
}

impl CoordinatorBuilder {
    pub fn new(ip: String, event_id: String, round: usize) -> Self {
        Self { ip, event_id, round}
    }
}

impl CoordinatorBuilder {
    pub async fn into_coordinator(self) -> Result<FlipUpVMixCoordinator, Error> {
        FlipUpVMixCoordinator::new(self.ip, self.event_id, 0, self.round).await
    }
}

impl From<Json<CoordinatorBuilder>> for CoordinatorBuilder {
    fn from(json: Json<CoordinatorBuilder>) -> Self {
        json.into_inner()
    }
}
