use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::queries::Division;
use crate::{controller, dto};
use itertools::Itertools;
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Debug, JsonSchema, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
    pub focused: bool,
    pub holes_finished: usize,
    pub index: usize,
    pub queue: Option<usize>,
    #[serde(skip)]
    pub division: Arc<Division>,
}
#[derive(Debug, JsonSchema, Clone, FromForm, Deserialize)]
pub struct HoleSetting {
    pub hole: Option<u8>,
    pub throws: Option<u8>,
}

impl Player {
    pub fn new(
        id: String,
        name: String,
        image_url: Option<String>,
        holes_finished: usize,
        index: usize,
        queue: Option<usize>,
        division: Arc<Division>,
    ) -> Self {
        Self {
            id,
            name,
            image_url,
            focused: false,
            holes_finished,
            index,
            queue,
            division,
        }
    }

    pub fn from_normal_player(player: controller::Player, queue: Option<usize>) -> Self {
        Player {
            id: player.player_id.clone(),
            name: player.name.clone(),
            image_url: player.image_url.clone(),
            focused: false,
            holes_finished: player.amount_of_holes_finished(),
            index: player.group_index,
            queue,
            division: player.division.clone(),
        }
    }
}
impl From<&controller::Player> for self::Player {
    fn from(player: &controller::Player) -> self::Player {
        self::Player {
            id: player.player_id.clone(),
            name: player.name.clone(),
            image_url: player.image_url.clone(),
            focused: false,
            holes_finished: player.amount_of_holes_finished(),
            index: player.group_index,
            queue: None,
            division: player.division.clone(),
        }
    }
}
