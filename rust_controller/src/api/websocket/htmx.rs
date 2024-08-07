use std::fmt::Debug;
use std::ops::Deref;

use rocket::futures::FutureExt;
use rocket::tokio::select;
use rocket::{Shutdown, State};
use rocket_dyn_templates::Metadata;
use rocket_ws as ws;
use rocket_ws::Message;
use serde_json::json;

pub use crate::api::websocket::channels::PlayerManagerUpdate;
use crate::api::websocket::channels::{DivisionUpdate, LeaderboardRoundUpdate};
use crate::api::websocket::{interpret_message, ChannelAttributes};
use crate::api::{Coordinator, GeneralChannel, HoleUpdate};
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::dto;

#[get("/players/selected/watch")]
pub fn focused_player_changer<'r>(
    ws: ws::WebSocket,
    coordinator: Coordinator,
    metadata: Metadata<'r>,
    player_updater: &'r GeneralChannel<PlayerManagerUpdate>,
    division_updater: &'r GeneralChannel<DivisionUpdate>,
) -> ws::Stream!['r] {
    let ws = ws.config(ws::Config {
        ..Default::default()
    });

    ws::Stream! { ws =>
        for await message in ws {
            let message = message?;
            if interpret_message(message.clone(), &coordinator,player_updater,division_updater).await.is_ok() {
                if let Some(html) = make_html_response::<PlayerManagerUpdate>(&coordinator,&metadata).await {
                    info!("Sending html response");
                    yield Message::from(html);
                }
            }
        }
    }
}

pub async fn general_htmx_updater<'r, T: ChannelAttributes + 'r>(
    ws: ws::WebSocket,
    coordinator: Coordinator,
    queue: &GeneralChannel<T>,
    metadata: Metadata<'r>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    use rocket::futures::SinkExt;

    let mut receiver = queue.subscribe();
    ws.channel(move |mut stream| Box::pin(async move {
        stream.send(make_html_response::<T>(&coordinator, &metadata).await.unwrap()).await.unwrap();
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

#[get("/players/selected/set")]
pub async fn selection_updater<'r>(
    ws: ws::WebSocket,
    coordinator: Coordinator,
    queue: &State<GeneralChannel<PlayerManagerUpdate>>,
    metadata: Metadata<'r>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    general_htmx_updater(ws, coordinator, queue, metadata, shutdown).await
}

#[get("/division")]
pub async fn division_updater<'r>(
    ws: ws::WebSocket,
    coordinator: Coordinator,
    queue: &State<GeneralChannel<DivisionUpdate>>,
    metadata: Metadata<'r>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    general_htmx_updater(ws, coordinator, queue, metadata, shutdown).await
}
async fn make_html_response<'r, T: ChannelAttributes>(
    coordinator: &Coordinator,
    metadata: &Metadata<'r>,
) -> Option<Message> {
    T::from(coordinator.lock().await.deref()).make_html(metadata)
}

#[get("/rounds")]
pub async fn leaderboard_round<'r>(
    ws: ws::WebSocket,
    coordinator: Coordinator,
    watcher: &State<GeneralChannel<LeaderboardRoundUpdate>>,
    metadata: Metadata<'r>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    general_htmx_updater(ws, coordinator, watcher, metadata, shutdown).await
}
