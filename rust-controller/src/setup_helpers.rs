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




macro_rules! make_js_async_fn {
    ($async_func:ident) => {
        #[wasm_bindgen]
        fn ($async_func + "test")() {
            let future = async move {
                $async_func()
            }
        }
    };
}


async fn test() {
    
}

make_js_async_fn!(test)


#[wasm_bindgen]
pub fn get_rounds_from_api(event_id: String) -> Promise {
    let future = async move {
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
    };
    future_to_promise(future)
}
