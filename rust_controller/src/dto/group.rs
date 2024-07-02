use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::Serialize;
use crate::dto;
#[derive(Serialize,JsonSchema)]
pub struct Group {
    players: Vec<dto::Player>,
    status: crate::controller::queries::group::GroupStatus,
}

impl From<&crate::controller::queries::Group> for Group {
    fn from(value: &crate::controller::queries::Group) -> Self {

        let players: Vec<dto::Player> = {
            value.player_connections_v2.iter().flat_map(|connection|{
                if let crate::controller::queries::group::GroupPlayerConnectionTypeCombined::GroupPlayerConnection(connection) = connection {
                    let player = &connection.player;
                    info!("using player {:?}",player.user);
                    let name = format!("{} {}", player.user.first_name.clone().unwrap(),player.user.last_name.clone().unwrap());
                    Some(dto::Player::new(player.id.clone().into_inner().parse().ok()?,name))
                } else {
                    None
                }
            }).collect()
        };
        info!("{:#?}",&players);
        Self {
            status: value.status,
            players
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
            status: value.status,
            players
        }
    }
}