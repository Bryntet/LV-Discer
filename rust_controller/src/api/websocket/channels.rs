use std::fmt::Debug;
use rocket::serde::Serialize;
use rocket_dyn_templates::Metadata;
use rocket_ws::Message;
use serde_json::json;
use tokio::sync::broadcast::error::SendError;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::dto;

pub struct GeneralChannel<T: for<'a> From<&'a FlipUpVMixCoordinator> + ChannelAttributes + Send + Clone + Debug> {
    sender: tokio::sync::broadcast::Sender<T>,
}
impl<T: for<'a> From<&'a FlipUpVMixCoordinator> + ChannelAttributes + Send + Clone + Debug> GeneralChannel<T> {
    pub fn send(&self, coordinator: &FlipUpVMixCoordinator) {
        let t = T::from(coordinator);
        match self.sender.send(t) {
            Ok(_) => (),
            Err(e) => warn!("Error sending message: {:?}", e),
        
        }
    }
    
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}

impl<T: for<'a> From<&'a FlipUpVMixCoordinator> + ChannelAttributes + Send + Clone + Debug> From<tokio::sync::broadcast::Sender<T>> for GeneralChannel<T> {
    fn from(sender: tokio::sync::broadcast::Sender<T>) -> Self {
        Self {
            sender,
        }
    }

}

#[derive(Debug, Clone, Serialize)]
pub struct GroupSelectionUpdate {
    players: Vec<dto::Player>,
}



pub trait ChannelAttributes {
    fn try_into_message(self) -> Option<Message>;
    fn make_html(self, metadata: &Metadata) -> Option<Message>;
}

impl ChannelAttributes for GroupSelectionUpdate {
    fn try_into_message(self) -> Option<Message> {
        Some(Message::from(serde_json::to_string(&self.players).ok()?))
    }
    
    fn make_html(self, metadata: &Metadata) -> Option<Message> {
        metadata.render("current_selected", json!({"players": self.players})).map(|(_,b)|Message::from(b))
    }
}

impl From<&FlipUpVMixCoordinator> for GroupSelectionUpdate {
    fn from(value: &FlipUpVMixCoordinator) -> Self {
        Self {
            players: dto::current_dto_players(value),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HoleUpdate {
    hole: usize,
}

impl ChannelAttributes for HoleUpdate {
    fn try_into_message(self) -> Option<Message> {
        Some(Message::from(serde_json::to_string(&self.hole).ok()?))
    }
    
    fn make_html(self, metadata: &Metadata) -> Option<Message> {
        metadata.render("current_hole", json!({"hole": self.hole})).map(|(_,b)|Message::from(b))
    }
}

impl From<&FlipUpVMixCoordinator> for HoleUpdate {
    fn from(value: &FlipUpVMixCoordinator) -> Self {
        Self {
            hole: value.current_hole(),
        }
    }
}