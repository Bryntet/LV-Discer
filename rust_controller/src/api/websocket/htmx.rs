pub use crate::api::websocket::channels::GroupSelectionUpdate;
use crate::api::websocket::{interpret_message, ChannelAttributes};
use crate::api::{Coordinator, GeneralChannel};
use crate::dto;
use rocket::futures::FutureExt;
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::Sender;
use rocket::{Shutdown, State};
use rocket_dyn_templates::{context, Metadata};
use rocket_ws as ws;
use rocket_ws::Message;
use serde::Deserialize;
use serde_json::json;
use std::fmt::Debug;


#[get("/players/selected/watch")]
pub fn focused_player_changer<'r>(
    ws: ws::WebSocket,
    coordinator: Coordinator,
    metadata: Metadata<'r>,
    updater: &'r State<GeneralChannel<GroupSelectionUpdate>>,
) -> ws::Stream!['r] {
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
pub async fn selection_updater<'r>(
    ws: ws::WebSocket,
    coordinator: Coordinator,
    queue: &State<GeneralChannel<GroupSelectionUpdate>>,
    metadata: Metadata<'r>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
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

async fn make_html_response<'r>(
    coordinator: &Coordinator,
    metadata: &Metadata<'r>,
) -> Option<String> {
    let c = coordinator.lock().await;
    metadata
        .render(
            "current_selected",
            json!({"players": dto::current_dto_players(&c)}),
        )
        .map(|(_, b)| b)
}
