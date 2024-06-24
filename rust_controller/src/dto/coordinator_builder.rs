use serde::Deserialize;
use crate::controller::coordinator::FlipUpVMixCoordinator;

use rocket_okapi::okapi::{schemars,schemars::JsonSchema};

#[derive(Default, Deserialize, JsonSchema)]
pub struct CoordinatorBuilder {
    ip: String,
    event_id: String,
    selected_division: String,
    focused_player: usize
}

impl From<CoordinatorBuilder> for FlipUpVMixCoordinator {
    fn from(builder: CoordinatorBuilder) -> Self {
        FlipUpVMixCoordinator::new(builder.ip, builder.event_id, builder.selected_division, builder.focused_player)
    }
}
