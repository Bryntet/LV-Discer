use rocket::serde::Serialize;
use rocket_dyn_templates::Metadata;
use rocket_ws::Message;
use serde_json::json;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::dto;

#[derive(Debug, Clone, Serialize)]
pub struct SelectionUpdate {
    players: Vec<dto::Player>,
}

impl SelectionUpdate {
    pub fn try_into_message(self) -> Option<Message> {
        Some(Message::from(serde_json::to_string(&self.players).ok()?))
    }
    
    pub fn make_html(self, metadata: &Metadata) -> Option<Message> {
        metadata.render("current_selected", json!({"players": self.players})).map(|(_,b)|Message::from(b))
    }
}

impl From<&FlipUpVMixCoordinator> for SelectionUpdate {
    fn from(value: &FlipUpVMixCoordinator) -> Self {
        Self {
            players: dto::current_dto_players(value),
        }
    }
}