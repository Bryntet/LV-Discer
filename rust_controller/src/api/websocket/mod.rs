mod channels;

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
use rocket::futures::stream::FusedStream;
use rocket::futures::TryStreamExt;
use rocket::response::stream::{EventStream, Event};
use rocket::tokio::sync::broadcast::{channel, Sender, Receiver};
use rocket::tokio::select;
use crate::controller::coordinator::FlipUpVMixCoordinator;

pub use channels::SelectionUpdate;

#[get("/player-selection-updater")]
pub async fn selection_updater<'r>(ws: ws::WebSocket, queue: &State<Sender<SelectionUpdate>>, metadata: Metadata<'r>) -> ws::Channel<'r> {
    use rocket::futures::{SinkExt, StreamExt};

    let mut receiver = queue.subscribe();
    ws.channel(move |mut stream| Box::pin(async move {
        loop {
            if stream.is_terminated() {
                break;
            }
            if let Ok(Some(message)) = receiver.recv().await.map(|update|update.make_html(&metadata)) {
                let _ = stream.send(message).await;
            }
        }
        stream.close(None).await
    }))
}

#[get("/selected-players")]
pub fn echo_stream<'r>(ws: ws::WebSocket, coordinator: Coordinator,metadata: Metadata<'r>, updater: &'r State<Sender<SelectionUpdate>>) -> ws::Stream!['r] {
    let ws = ws.config(ws::Config {
        ..Default::default()
    });


    ws::Stream! { ws =>
        for await message in ws {
            let message = message?;

            if interpret_message(message.clone(), &coordinator,updater).await.is_ok() {
                if let Some(html) = make_html_response(&coordinator,&metadata).await {
                    info!("Sending html response");
                    yield Message::from(html);
                }
            }
        }
    }
}

async fn interpret_message<'r>(message: Message, coordinator: &Coordinator, updater: &State<Sender<SelectionUpdate>>) -> Result<Interpreter, serde_json::Error> {
    let interpreter: Interpreter = serde_json::from_str(&message.to_string())?;
    if let Ok(num) = interpreter.message.parse::<usize>() {
        let mut c = coordinator.lock().await;
        c.set_focused_player(num, Some(updater));
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