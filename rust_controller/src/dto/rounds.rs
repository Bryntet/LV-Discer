use itertools::Itertools;
use crate::controller::get_data;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::Serialize;
use crate::api::MyError;

#[derive(Serialize, JsonSchema)]
pub struct SimpleRound {
    round: usize,
    id: String,
}
impl SimpleRound {
    pub fn new(round: usize, id: String) -> Self {
        Self { round, id }
    }
}

#[derive(Serialize, JsonSchema)]
pub struct Rounds(Vec<SimpleRound>);

pub async fn get_rounds(event_id: String) -> Result<Rounds,MyError> {
    let time = std::time::Instant::now();
    let ids = get_data::RustHandler::get_rounds(&event_id).await?;
    info!("Time to get rounds: {:?}", time.elapsed());
    Ok(Rounds(ids.into_iter().enumerate().map(|(i, round_id)| SimpleRound::new(i, round_id.to_string())).collect_vec()))
}
