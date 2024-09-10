use itertools::Itertools;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::Serialize;

use crate::api::Error;
use crate::controller::get_data;

#[derive(Serialize, JsonSchema, Debug, Clone)]
pub struct SimpleRound {
    number: usize,
    id: [String; 3],
    selected: bool,
}
impl SimpleRound {
    pub fn new(round: usize, id: [String; 3], selected: bool) -> Self {
        Self {
            number: round + 1,
            id,
            selected,
        }
    }
}

#[derive(Serialize, JsonSchema)]
pub struct Rounds(Vec<SimpleRound>);
