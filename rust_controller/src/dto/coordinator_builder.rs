use rocket::serde::json::Json;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::Deserialize;

use crate::api::Error;
use crate::controller::coordinator::FlipUpVMixCoordinator;

#[derive(Default, Deserialize, JsonSchema, FromForm)]
pub struct CoordinatorBuilder {
    ip: String,
    event_id: String,
    round: usize,
    featured_hole: u8,
}

impl CoordinatorBuilder {
    pub fn new(ip: String, event_id: String, round: usize, featured_hole: u8) -> Self {
        Self {
            ip,
            event_id,
            round,
            featured_hole,
        }
    }
}

impl CoordinatorBuilder {
    pub async fn into_coordinator(self) -> Result<FlipUpVMixCoordinator, Error> {
        FlipUpVMixCoordinator::new(self.ip, self.event_id, 0, self.round, self.featured_hole).await
    }
}

impl From<Json<CoordinatorBuilder>> for CoordinatorBuilder {
    fn from(json: Json<CoordinatorBuilder>) -> Self {
        json.into_inner()
    }
}
