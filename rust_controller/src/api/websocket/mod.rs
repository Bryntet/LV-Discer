mod channels;
pub mod htmx;

use std::fmt::Debug;
use std::ops::Deref;
use rocket_dyn_templates::Metadata;
use rocket_ws as ws;
use rocket_ws::Message;
use serde::Deserialize;
use serde_json::json;
use crate::api::Coordinator;
use crate::dto;
use rocket::{State, Shutdown};
use rocket::futures::FutureExt;
use rocket::serde::json::Json;
use rocket::tokio::sync::broadcast::Sender;
use rocket::tokio::select;

pub use channels::{GroupSelectionUpdate, ChannelAttributes};

async fn interpret_message<'r>(message: Message, coordinator: &Coordinator, updater: &State<Sender<GroupSelectionUpdate>>) -> Result<Interpreter, serde_json::Error> {
    let interpreter: Interpreter = serde_json::from_str(&message.to_string())?;
    if let Ok(num) = interpreter.message.parse::<usize>() {
        let mut c = coordinator.lock().await;
        c.set_focused_player(num, Some(updater));
    }
    Ok(interpreter)
}

#[get("/players/selected/watch")]
pub async fn selection_updater<'r>(ws: ws::WebSocket, queue: &State<Sender<GroupSelectionUpdate>>, metadata: Metadata<'r>, shutdown: Shutdown) -> ws::Channel<'r> {
    use rocket::futures::SinkExt;

    let mut receiver = queue.subscribe();
    ws.channel(move |mut stream| Box::pin(async move {

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

    }))
}


#[derive(Deserialize, Debug)]
struct Interpreter {
    message: String,
    
}