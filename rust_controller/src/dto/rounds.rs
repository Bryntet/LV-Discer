use itertools::Itertools;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::Serialize;

use crate::api::Error;
use crate::controller::get_data;

#[derive(Serialize, JsonSchema, Debug, Clone)]
pub struct SimpleRound {
    number: usize,
    id: String,
    selected: bool,
}
impl SimpleRound {
    pub fn new(round: usize, id: String, selected: bool) -> Self {
        Self {
            number: round + 1,
            id,
            selected,
        }
    }
}

#[derive(Serialize, JsonSchema)]
pub struct Rounds(Vec<SimpleRound>);

pub async fn get_rounds(event_id: String) -> Result<Rounds, Error> {
    let time = std::time::Instant::now();
    let ids = get_data::RustHandler::get_rounds(&event_id).await?;
    let amount_of_rounds = ids.len();
    info!("Time to get rounds: {:?}", time.elapsed());
    Ok(Rounds(
        ids.into_iter()
            .enumerate()
            .map(|(i, round_id)| {
                SimpleRound::new(i, round_id.to_string(), i + 1 == amount_of_rounds)
            })
            .collect_vec(),
    ))
}
