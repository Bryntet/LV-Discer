use crate::controller::get_data;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::Serialize;

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

pub async fn get_rounds(event_id: String) -> Rounds {
    let request = get_data::post_status(event_id.into()).await;
    Rounds(
        request
            .data
            .and_then(|t| {
                t.event.map(|event| {
                    event
                        .rounds
                        .into_iter()
                        .flatten()
                        .enumerate()
                        .map(|(round_number, round)| SimpleRound {
                            round: round_number,
                            id: round.id.into_inner(),
                        })
                        .collect::<Vec<_>>()
                })
            })
            .unwrap(),
    )
}
