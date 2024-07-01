use crate::controller::coordinator::FlipUpVMixCoordinator;
use serde::Deserialize;

use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use crate::api::MyError;

#[derive(Default, Deserialize, JsonSchema)]
pub struct CoordinatorBuilder {
    ip: String,
    event_id: String,
    focused_player: usize,
}
impl CoordinatorBuilder {
    pub async fn into_coordinator(self) -> Result<FlipUpVMixCoordinator, MyError> {
        FlipUpVMixCoordinator::new(
            self.ip,
            self.event_id,
            self.focused_player
        ).await
    }
}

