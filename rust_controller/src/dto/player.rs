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
    pub holes_finished: usize,
    pub index: usize,
}

impl Player {
    pub fn new(id: String, name: String, image_url: Option<String>, holes_finished: usize, index:usize) -> Self {
        Self { id, name, image_url, focused: false, holes_finished,index}
    }
}
impl From<&controller::Player> for self::Player {
    fn from(value: &controller::Player) -> self::Player {
        self::Player {
            id: value.player_id.clone(),
            name: value.name.clone(),
            image_url: value.image_url.clone(),
            focused: false,
            holes_finished: value.amount_of_holes_finished(),
            index: value.ind,
        }
    }
}

pub fn current_dto_players(coordinator: &FlipUpVMixCoordinator) -> Vec<dto::Player> {
    let mut players: Vec<dto::Player> = coordinator.current_players().into_iter().map(dto::Player::from).collect_vec();
    for (ind,player) in &mut players.iter_mut().enumerate() {
        player.index = ind;
        if player.id == coordinator.focused_player().player_id {
            player.focused = true;
        }
    }
    players
}
