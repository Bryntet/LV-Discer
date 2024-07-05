use itertools::Itertools;
use crate::{controller, dto};
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::Serialize;
use crate::controller::coordinator::FlipUpVMixCoordinator;

#[derive(Serialize, Debug, JsonSchema, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
    pub focused: bool,
}

impl Player {
    pub fn new(id: String, name: String, image_url: Option<String>) -> Self {
        Self { id, name, image_url, focused: false}
    }
}
impl From<&controller::Player> for self::Player {
    fn from(value: &controller::Player) -> self::Player {
        self::Player {
            id: value.player_id.clone(),
            name: value.name.clone(),
            image_url: value.image_url.clone(),
            focused: false,
        }
    }
}

pub fn current_dto_players(coordinator: &FlipUpVMixCoordinator) -> Vec<dto::Player> {
    let mut players: Vec<dto::Player> = coordinator.current_players().into_iter().map(dto::Player::from).collect_vec();
    if let Some(focused_player) = players.iter_mut().find(|player|player.id==coordinator.focused_player().player_id) {
        focused_player.focused = true;
    }
    players
}
