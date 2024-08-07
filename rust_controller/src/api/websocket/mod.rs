use std::fmt::Debug;

use rocket::futures::FutureExt;
use rocket::tokio::select;
use rocket::{Shutdown, State};
use rocket_ws as ws;
use rocket_ws::Message;
use serde::Deserialize;

pub use channels::{
    ChannelAttributes, DivisionUpdate, LeaderboardRoundUpdate, PlayerManagerUpdate,
};

use crate::api::websocket::channels::{GeneralChannel, HoleUpdate};
use crate::api::Coordinator;
use crate::controller::coordinator::FlipUpVMixCoordinator;

pub mod channels;
pub mod htmx;

#[inline(always)]
async fn interpret_message(
    message: Message,
    coordinator: &Coordinator,
    player_updater: &GeneralChannel<PlayerManagerUpdate>,
    division_updater: &GeneralChannel<DivisionUpdate>,
) -> Result<Interpreter, serde_json::Error> {
    let interpreter: Interpreter = serde_json::from_str(&message.to_string())?;
    if let Ok(num) = interpreter.message.parse::<usize>() {
        let mut c = coordinator.lock().await;
        c.set_focused_player(num, player_updater, division_updater);
    }
    Ok(interpreter)
}

pub async fn make_watcher_websocket<
    'r,
    T: for<'a> From<&'a FlipUpVMixCoordinator> + ChannelAttributes + Send + Clone + Debug,
>(
    ws: ws::WebSocket,
    coordinator: &'r State<GeneralChannel<T>>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    use rocket::futures::SinkExt;

    let mut receiver = coordinator.subscribe();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            loop {
                select! {
                    message = receiver.recv().fuse() => {
                        if let Ok(Some(message)) = message.map(|update|update.try_into_message()) {
                            let _ = stream.send(message).await;
                        }
                    },
                    _ = shutdown.clone().fuse() => break,
                }
            }
            Ok(())
        })
    })
}

#[get("/players/selected/watch")]
pub async fn selection_watcher<'r>(
    ws: ws::WebSocket,
    queue: &'r State<GeneralChannel<PlayerManagerUpdate>>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    make_watcher_websocket(ws, queue, shutdown).await
}

#[get("/hole/watch")]
pub async fn hole_watcher(
    ws: ws::WebSocket,
    hole_watcher: &State<GeneralChannel<HoleUpdate>>,
    shutdown: Shutdown,
) -> ws::Channel {
    make_watcher_websocket(ws, hole_watcher, shutdown).await
}

#[get("/rounds")]
pub async fn leaderboard_round_watcher(
    ws: ws::WebSocket,
    watcher: &State<GeneralChannel<LeaderboardRoundUpdate>>,
    shutdown: Shutdown,
) -> ws::Channel {
    make_watcher_websocket(ws, watcher, shutdown).await
}
#[derive(Deserialize, Debug)]
struct Interpreter {
    message: String,
}
