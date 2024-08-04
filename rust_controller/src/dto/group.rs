use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::Serialize;

use crate::dto;
use crate::dto::Player;

#[derive(Serialize, JsonSchema, Clone, Debug)]
pub struct Group {
    pub players: Vec<dto::Player>,
    pub id: String,
    pub group_number: usize,
    pub start_at_hole: u8,
    pub start_time: Option<u32>,
}

impl Group {
    pub fn new(
        id: String,
        players: Vec<Player>,
        group_number: usize,
        start_at: u8,
        date_time: Option<DateTime<Utc>>,
    ) -> Self {
        Group {
            players,
            id,
            group_number,
            start_at_hole: start_at,
            start_time: date_time.map(|date| date.time().num_seconds_from_midnight()),
        }
    }

    pub fn player_ids(&self) -> Vec<String> {
        self.players
            .iter()
            .cloned()
            .map(|player| player.id)
            .collect()
    }
}
impl From<&crate::controller::queries::Group> for Group {
    fn from(group: &crate::controller::queries::Group) -> Self {
        let players: Vec<dto::Player> = {
            group.player_connections_v2.iter().flat_map(|connection|{
                if let crate::controller::queries::group::GroupPlayerConnectionTypeCombined::GroupPlayerConnection(connection) = connection {
                    let player = &connection.player;
                    let name = format!("{} {}", player.user.first_name.clone().unwrap(),player.user.last_name.clone().unwrap());
                    Some(dto::Player::new(player.id.clone().into_inner(),name, None,0,100, None))
                } else {
                    None
                }
            }).collect()
        };
        Self {
            players,
            id: group.id.clone().into_inner(),
            group_number: group.position as usize + 1,
            start_at_hole: group
                .start_hole
                .clone()
                .map(|hole| hole.number as u8)
                .unwrap_or(0),
            start_time: group.starts_at.map(|date| date.num_seconds_from_midnight()),
        }
    }
}

impl From<crate::controller::queries::Group> for Group {
    fn from(group: crate::controller::queries::Group) -> Self {
        let players: Vec<dto::Player> = {
            group.player_connections_v2.into_iter().flat_map(|connection|{
                if let crate::controller::queries::group::GroupPlayerConnectionTypeCombined::GroupPlayerConnection(connection) = connection {
                    let player = connection.player;

                    let name = format!("{} {}", player.user.first_name.unwrap(),player.user.last_name.unwrap());
                    Some(dto::Player::new(player.id.into_inner().parse().ok()?,name,None,0, 100,None))
                } else {
                    None
                }
            }).collect()
        };
        Self {
            players,
            id: group.id.into_inner(),
            group_number: group.position as usize + 1,
            start_at_hole: group.start_hole.map(|hole| hole.number as u8).unwrap_or(0),
            start_time: group.starts_at.map(|time| time.num_seconds_from_midnight()),
        }
    }
}
