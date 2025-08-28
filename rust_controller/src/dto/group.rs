use crate::controller::queries::layout::Layout;
use crate::controller::queries::Division;
use crate::dto;
use crate::dto::Player;
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, JsonSchema, Clone, Debug)]
pub struct Group {
    pub players: Vec<dto::Player>,
    pub id: String,
    pub group_number: usize,
    pub start_at_hole: u8,
    pub start_time: Option<u32>,
    #[serde(skip)]
    pub layout: Arc<Layout>,
}

impl Group {
    pub fn new(
        id: String,
        players: Vec<Player>,
        group_number: usize,
        start_at: u8,
        date_time: Option<DateTime<Utc>>,
        layout: Arc<Layout>,
    ) -> Self {
        Group {
            players,
            id,
            group_number,
            start_at_hole: start_at,
            start_time: date_time.map(|date| date.time().num_seconds_from_midnight()),
            layout,
        }
    }

    pub fn player_ids(&self) -> Vec<String> {
        self.players
            .iter()
            .map(|player| player.id.to_string())
            .collect()
    }
}

impl crate::controller::queries::Group {
    pub fn to_dto_group(self, layout: Arc<Layout>) -> Group {
        let players: Vec<dto::Player> = {
            self.player_connections_v2.into_iter().flat_map(|connection| {
                if let crate::controller::queries::group::GroupPlayerConnectionTypeCombined::GroupPlayerConnection(connection) = connection {
                    let player = connection.player;

                    let name = format!("{} {}", player.user.first_name.unwrap(), player.user.last_name.unwrap());
                    Some(dto::Player::new(player.id.into_inner().parse().ok()?, name, None, 0, 100, None, Arc::new(Division{
                        id: cynic::Id::new(""),
                        short_name: "MPO".to_string(),
                        name: "Mixed Pro Open".to_string(),
                    }) ))
                } else {
                    None
                }
            }).collect()
        };
        Group {
            players,
            id: self.id.into_inner(),
            group_number: self.position as usize + 1,
            start_at_hole: self.start_hole.map(|hole| hole.number as u8).unwrap_or(0),
            start_time: self.starts_at.map(|time| time.num_seconds_from_midnight()),
            layout,
        }
    }
}
