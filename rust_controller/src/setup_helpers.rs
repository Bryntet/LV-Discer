use cynic::GraphQlResponse;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::get_data;
use wasm_bindgen_futures::future_to_promise;
use js_sys::Promise;

#[wasm_bindgen]
struct SimpleRound {
    round: usize,
    id: String,
}
#[wasm_bindgen]
impl SimpleRound {
    #[wasm_bindgen(constructor)]
    pub fn new(round: usize, id: String) -> Self {
        Self {
            round,id
        }
    }
}

#[wasm_bindgen]
#[allow(unused)]
struct Rounds(Vec<SimpleRound>);









#[wasm_macros::wasm_async]
pub async fn get_rounds_from_api(event_id: String) -> Result<JsValue, JsValue> {
    let request = get_data::post_status(event_id.into()).await;
    match request.data.and_then(|t| {
        t.event.map(|event| {
            event.rounds.into_iter().flatten().enumerate()
                .map(|(round_number, round)| SimpleRound {
                    round: round_number,
                    id: round.id.into_inner(),
                }).collect::<Vec<_>>()
        })
    }) {
        None => Err(JsValue::from_str("AAA")),
        Some(rounds) => Ok(JsValue::from(Rounds(rounds)))
    }
}
