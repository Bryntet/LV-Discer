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

pub use crate::api::websocket::channels::SelectionUpdate;
use crate::api::websocket::{interpret_message};

#[get("/players/selected/watch")]
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


#[get("/players/selected/set")]
pub async fn selection_updater<'r>(ws: ws::WebSocket,coordinator: Coordinator, queue: &State<Sender<SelectionUpdate>>, metadata: Metadata<'r>, shutdown: Shutdown) -> ws::Channel<'r> {
    use rocket::futures::SinkExt;

    let mut receiver = queue.subscribe();
    ws.channel(move |mut stream| Box::pin(async move {
        stream.send(Message::from(make_html_response(&coordinator,&metadata).await.unwrap())).await;
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

async fn make_html_response<'r>(coordinator: &Coordinator, metadata: &Metadata<'r>) -> Option<String> {
    let c = coordinator.lock().await;
    metadata.render("current_selected", json!({"players": dto::current_dto_players(&c)})).map(|(_,b)|b)
}