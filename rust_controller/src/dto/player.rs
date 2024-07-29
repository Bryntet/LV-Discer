use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::{controller, dto};
use itertools::Itertools;
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, Debug, JsonSchema, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
    pub focused: bool,
    pub holes_finished: usize,
    pub index: usize,
    pub queue: Option<usize>,
}

impl Player {
    pub fn new(
        id: String,
        name: String,
        image_url: Option<String>,
        holes_finished: usize,
        index: usize,
        queue: Option<usize>,
    ) -> Self {
        Self {
            id,
            name,
            image_url,
            focused: false,
            holes_finished,
            index,
            queue,
        }
    }

    pub fn from_normal_player(player: controller::Player, queue: Option<usize>) -> Self {
        Player {
            id: player.player_id.clone(),
            name: player.name.clone(),
            image_url: player.image_url.clone(),
            focused: false,
            holes_finished: player.amount_of_holes_finished(),
            index: player.ind,
            queue,
        }
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
            queue: None,
        }
    }
}
