mod channels;

use std::fmt::Debug;
use rocket_dyn_templates::Metadata;
use rocket_ws as ws;
use rocket_ws::Message;
use serde::Deserialize;
use serde_json::json;
use crate::api::Coordinator;
use crate::dto;
use rocket::{State, Shutdown};
use rocket::futures::FutureExt;
use rocket::tokio::sync::broadcast::Sender;
use rocket::tokio::select;

pub use channels::SelectionUpdate;

#[get("/player-selection-updater")]
pub async fn selection_updater<'r>(ws: ws::WebSocket, queue: &State<Sender<SelectionUpdate>>, metadata: Metadata<'r>, shutdown: Shutdown) -> ws::Channel<'r> {
    use rocket::futures::SinkExt;

    let mut receiver = queue.subscribe();
    ws.channel(move |mut stream| Box::pin(async move {

        loop {
            select! {
                message = receiver.recv().fuse() => {
                    if let Ok(Some(message)) = message.map(|update|update.make_html(&metadata)) {
                        let _ = stream.send(message).await;
                    }
                },
                _ = shutdown.clone().fuse() => break,
            }
        }
        Ok(())

    }))
}
#[get("/selected-players")]
pub fn focused_player_changer<'r>(ws: ws::WebSocket, coordinator: Coordinator,metadata: Metadata<'r>, updater: &'r State<Sender<SelectionUpdate>>) -> ws::Stream!['r] {
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
    let c = coordinator.lock().await;
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