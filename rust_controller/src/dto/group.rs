use std::collections::HashMap;
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::Serialize;
use crate::dto;
use crate::dto::Player;

#[derive(Serialize,JsonSchema, Clone)]
pub struct Group {
    pub players: Vec<dto::Player>,
    pub id: String
}
impl Group {
    pub fn new(id: String, players: Vec<Player>) -> Self {
        Group {
            players,
            id
        }
    }
    
    pub fn player_ids(&self) -> Vec<String> {
        self.players.iter().map(|player|player.id.clone()).collect()
    }
}
impl From<&crate::controller::queries::Group> for Group {
    fn from(value: &crate::controller::queries::Group) -> Self {

        let players: Vec<dto::Player> = {
            value.player_connections_v2.iter().flat_map(|connection|{
                if let crate::controller::queries::group::GroupPlayerConnectionTypeCombined::GroupPlayerConnection(connection) = connection {
                    let player = &connection.player;
                    let name = format!("{} {}", player.user.first_name.clone().unwrap(),player.user.last_name.clone().unwrap());
                    Some(dto::Player::new(player.id.clone().into_inner().parse().ok()?,name))
                } else {
                    None
                }
            }).collect()
        };
        Self {
            players,
            id: value.id.clone().into_inner()
        }
    }
}





impl From<crate::controller::queries::Group> for Group {
    fn from(value: crate::controller::queries::Group) -> Self {
        let players: Vec<dto::Player> = {
            value.player_connections_v2.into_iter().flat_map(|connection|{
                if let crate::controller::queries::group::GroupPlayerConnectionTypeCombined::GroupPlayerConnection(connection) = connection {
                    let player = connection.player;

                    let name = format!("{} {}", player.user.first_name.unwrap(),player.user.last_name.unwrap());
                    Some(dto::Player::new(player.id.into_inner().parse().ok()?,name))
                } else {
                    None
                }
            }).collect()
        };
        Self {
            players,
            id: value.id.into_inner()
        }
    }
}