use itertools::Itertools;
use rocket::serde::Serialize;
use rocket_dyn_templates::Metadata;
use rocket_ws::Message;
use serde_json::json;
use std::fmt::Debug;
use std::sync::Arc;

use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::dto;
use crate::dto::Division;
#[derive(Clone)]
pub struct GeneralChannel<T: ChannelAttributes> {
    sender: Arc<tokio::sync::broadcast::Sender<T>>,
}
impl<T: ChannelAttributes> GeneralChannel<T> {
    pub fn send_from_coordinator(&self, coordinator: &FlipUpVMixCoordinator) {
        let t = T::from(coordinator);
        self.send(t)
    }

    pub fn send(&self, t: T) {
        match self.sender.send(t) {
            Ok(_) => (),
            Err(e) => warn!("Error sending message: {:?}", e),
        }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}

impl<T: ChannelAttributes> From<tokio::sync::broadcast::Sender<T>> for GeneralChannel<T> {
    fn from(sender: tokio::sync::broadcast::Sender<T>) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerManagerUpdate {
    players: Vec<dto::Player>,
}

pub trait ChannelAttributes:
    for<'a> From<&'a FlipUpVMixCoordinator> + Send + Clone + Debug
{
    fn try_into_message(self) -> Option<Message>;
    fn make_html(self, metadata: &Metadata) -> Option<Message>;
}

impl ChannelAttributes for PlayerManagerUpdate {
    fn try_into_message(self) -> Option<Message> {
        Some(Message::from(serde_json::to_string(&self.players).ok()?))
    }

    fn make_html(self, metadata: &Metadata) -> Option<Message> {
        metadata
            .render("current_selected", json!({"players": self.players}))
            .map(|(_, b)| Message::from(b))
    }
}

impl From<&FlipUpVMixCoordinator> for PlayerManagerUpdate {
    fn from(coordinator: &FlipUpVMixCoordinator) -> Self {
        Self {
            players: coordinator.dto_players(),
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
        metadata
            .render("current_hole", json!({"hole": self.hole}))
            .map(|(_, b)| Message::from(b))
    }
}

impl From<&FlipUpVMixCoordinator> for HoleUpdate {
    fn from(value: &FlipUpVMixCoordinator) -> Self {
        Self {
            hole: value.current_hole() + 1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DivisionUpdate {
    divisions: Vec<dto::Division>,
}

impl ChannelAttributes for DivisionUpdate {
    fn try_into_message(self) -> Option<Message> {
        Some(Message::from(serde_json::to_string(&self.divisions).ok()?))
    }

    fn make_html(self, metadata: &Metadata) -> Option<Message> {
        metadata
            .render("divisions", json!({"divisions": self.divisions}))
            .map(|(_, b)| Message::from(b))
    }
}

impl From<&FlipUpVMixCoordinator> for DivisionUpdate {
    fn from(coordinator: &FlipUpVMixCoordinator) -> Self {
        Self {
            divisions: coordinator
                .all_divs
                .iter()
                .map(|div| {
                    if div.id == coordinator.leaderboard_division.id {
                        Division {
                            name: div.name.clone(),
                            focused: true,
                        }
                    } else {
                        Division {
                            name: div.name.clone(),
                            focused: false,
                        }
                    }
                })
                .collect_vec(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LeaderboardRoundUpdate {
    rounds: Vec<dto::SimpleRound>,
}

impl From<&FlipUpVMixCoordinator> for LeaderboardRoundUpdate {
    fn from(coordinator: &FlipUpVMixCoordinator) -> Self {
        Self {
            rounds: coordinator.dto_rounds(),
        }
    }
}

impl ChannelAttributes for LeaderboardRoundUpdate {
    fn try_into_message(self) -> Option<Message> {
        Some(Message::from(serde_json::to_string(&self.rounds).ok()?))
    }

    fn make_html(self, metadata: &Metadata) -> Option<Message> {
        metadata
            .render("rounds", json!({"rounds":self.rounds}))
            .map(|(_, message)| Message::from(message))
    }
}

#[derive(Clone, Debug)]
pub enum HoleFinishedAlert {
    JustFinished,
    SecondSend,
}

impl From<&FlipUpVMixCoordinator> for HoleFinishedAlert {
    fn from(value: &FlipUpVMixCoordinator) -> Self {
        Self::JustFinished
    }
}
impl ChannelAttributes for HoleFinishedAlert {
    fn try_into_message(self) -> Option<Message> {
        Some(Message::from(match self {
            HoleFinishedAlert::JustFinished => "NU",
            HoleFinishedAlert::SecondSend => "SEN",
        }))
    }
    fn make_html(self, metadata: &Metadata) -> Option<Message> {
        None
    }
}
