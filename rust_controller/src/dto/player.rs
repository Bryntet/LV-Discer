use crate::controller;
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::Serialize;
#[derive(Serialize, Debug, JsonSchema, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
}

impl Player {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}
impl From<&controller::Player> for self::Player {
    fn from(value: &controller::Player) -> self::Player {
        self::Player {
            id: value.player_id.clone(),
            name: value.name.clone(),
        }
    }
}
