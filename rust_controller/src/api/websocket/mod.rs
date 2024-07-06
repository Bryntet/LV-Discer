pub mod channels;
pub mod htmx;

use crate::api::Coordinator;
use crate::dto;
use rocket::futures::FutureExt;
use rocket::serde::json::Json;
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::Sender;
use rocket::{Shutdown, State};
use rocket_dyn_templates::Metadata;
use rocket_ws as ws;
use rocket_ws::Message;
use serde::Deserialize;
use serde_json::json;
use std::fmt::Debug;
use std::ops::Deref;

use crate::api::websocket::channels::{GeneralChannel, HoleUpdate};
use crate::controller::coordinator::FlipUpVMixCoordinator;
pub use channels::{ChannelAttributes, GroupSelectionUpdate};

async fn interpret_message<'r>(
    message: Message,
    coordinator: &Coordinator,
    updater: &State<GeneralChannel<GroupSelectionUpdate>>,
) -> Result<Interpreter, serde_json::Error> {
    let interpreter: Interpreter = serde_json::from_str(&message.to_string())?;
    if let Ok(num) = interpreter.message.parse::<usize>() {
        let mut c = coordinator.lock().await;
        c.set_focused_player(num, Some(updater));
    }
    Ok(interpreter)
}

#[get("/players/selected/watch")]
pub async fn selection_watcher<'r>(
    ws: ws::WebSocket,
    queue: &'r State<GeneralChannel<GroupSelectionUpdate>>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    make_watcher_websocket(ws, queue, shutdown).await
}

#[get("/hole/watch")]
pub async fn hole_watcher<'r>(
    ws: ws::WebSocket,
    hole_watcher: &'r State<GeneralChannel<HoleUpdate>>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    make_watcher_websocket(ws, hole_watcher, shutdown).await
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

#[derive(Deserialize, Debug)]
struct Interpreter {
    message: String,
}
