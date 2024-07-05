
use std::fmt::Debug;
use rocket::{Orbit, Rocket};
use rocket_dyn_templates::{context, Metadata, Template};
use rocket_okapi::okapi::schemars::_private::NoSerialize;
use rocket_okapi::openapi;
use rocket_ws as ws;
use rocket_ws::Message;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::api::Coordinator;
use crate::dto;
use rocket::{State, Shutdown};
use rocket::response::stream::{EventStream, Event};
use rocket::tokio::sync::broadcast::{channel, Sender, Receiver};
use rocket::tokio::select;
use crate::controller::coordinator::FlipUpVMixCoordinator;

#[derive(Debug, Clone, Serialize)]
pub struct SelectionUpdate {
    players: Vec<dto::Player>,
}

impl From<&FlipUpVMixCoordinator> for SelectionUpdate {
    fn from(value: &FlipUpVMixCoordinator) -> Self {
        Self {
            players: dto::current_dto_players(value),
        }
    }
}

#[get("/player-selection-updater")]
pub async fn selection_updater(queue: &State<Sender<SelectionUpdate>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(_) => break,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}

#[get("/selected-players")]
pub fn echo_stream(ws: ws::WebSocket, coordinator: Coordinator,metadata: Metadata<'_>) -> ws::Stream!['_] {
    let ws = ws.config(ws::Config {
        ..Default::default()
    });


    ws::Stream! { ws =>
        for await message in ws {
            let message = message?;

            if interpret_message(message.clone(), &coordinator).await.is_ok() {
                if let Some(html) = make_html_response(&coordinator,&metadata).await {
                    info!("Sending html response");
                    yield Message::from(html);
                }
            }
        }
    }
}

async fn interpret_message<'r>(message: Message, coordinator: &Coordinator) -> Result<Interpreter, serde_json::Error> {
    let interpreter: Interpreter = serde_json::from_str(&message.to_string())?;
    if let Ok(num) = interpreter.message.parse::<usize>() {
        let mut c = coordinator.lock().await;
        c.set_focused_player(num);
    }
    Ok(interpreter)

}

async fn make_html_response<'r>(coordinator: &Coordinator, metadata: &Metadata<'r>) -> Option<String> {
    let mut c = coordinator.lock().await;
     metadata.render("current_selected", json!({"players": dto::current_dto_players(&c)})).map(|(_,b)|b)

}
#[derive(Deserialize, Debug)]
struct Interpreter {
    message: String,
    #[serde(rename = "HEADERS")]
    headers: HtmxHeaders,
}
#[derive(Deserialize, Debug)]
struct HtmxHeaders {
    #[serde(rename = "HX-Current-URL")]
    current_url: String,
    #[serde(rename = "HX-Request")]
    request: String,
    #[serde(rename = "HX-Trigger")]
    trigger: String,
    #[serde(rename = "HX-Trigger-Name")]
    trigger_name: Option<String>,
}