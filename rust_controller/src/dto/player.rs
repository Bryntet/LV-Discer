use crate::controller;
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::Serialize;
#[derive(Serialize, JsonSchema)]
pub struct Player {
    division: Division,
    pdga_num: u32,
    name: String,
}

impl From<&controller::Player> for self::Player {
    fn from(value: &controller::Player) -> self::Player {
        // TODO: Make not fixed division
        self::Player {
            division: Division::MPO,
            pdga_num: value.player_id.parse().unwrap(),
            name: value.name.clone(),
        }
    }
}
#[derive(Serialize, JsonSchema)]
pub enum Division {
    MPO,
    FPO,
}
