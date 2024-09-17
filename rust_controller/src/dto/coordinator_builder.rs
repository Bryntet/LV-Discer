use crate::api::Error;
use crate::controller::coordinator::{BroadcastType, FlipUpVMixCoordinator};
use itertools::Itertools;
use rocket::serde::json::Json;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default, Deserialize, JsonSchema, FromForm)]
pub struct CoordinatorBuilder {
    ip: String,
    event_ids: Vec<String>,
    round: usize,
    featured_hole: u8,
    broadcast_type: BroadcastType,
}

impl CoordinatorBuilder {
    pub fn new(
        ip: String,
        event_ids: Vec<String>,
        round: usize,
        featured_hole: u8,
        broadcast_type: BroadcastType,
    ) -> Self {
        Self {
            ip,
            event_ids,
            round,
            featured_hole,
            broadcast_type,
        }
    }
}

impl CoordinatorBuilder {
    pub async fn into_coordinator(self) -> Result<FlipUpVMixCoordinator, Error> {
        std::fs::write(
            Path::new("previous_ids.txt"),
            self.event_ids
                .iter()
                .map(|s| s.to_string() + "\n")
                .collect::<String>(),
        )
        .unwrap();
        FlipUpVMixCoordinator::new(
            self.ip,
            self.event_ids,
            0,
            self.round,
            self.featured_hole,
            self.broadcast_type,
        )
        .await
    }
}

impl From<Json<CoordinatorBuilder>> for CoordinatorBuilder {
    fn from(json: Json<CoordinatorBuilder>) -> Self {
        json.into_inner()
    }
}
