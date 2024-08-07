pub use group::Group;

#[derive(cynic::QueryVariables, Debug)]
pub struct RoundResultsQueryVariables {
    pub event_id: cynic::Id,
    pub round_id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "RootQuery", variables = "RoundResultsQueryVariables")]
pub struct RoundResultsQuery {
    #[arguments(eventId: $event_id)]
    pub event: Option<Event>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "RoundResultsQueryVariables")]
pub struct Event {
    pub players: Vec<Player>,
    pub divisions: Vec<Option<Division>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Division {
    pub name: String,
    pub id: cynic::Id,
}

impl Default for Division {
    fn default() -> Self {
        Division {
            name: "".to_string(),
            id: cynic::Id::new(""),
        }
    }
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "RoundResultsQueryVariables")]
pub struct Player {
    pub user: User,
    pub dnf: Dnf,
    pub dns: Dns,
    pub division: Division,
    #[arguments(roundId: $round_id)]
    pub results: Option<Vec<HoleResult>>,
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Result")]
pub struct HoleResult {
    pub hole: Hole,
    pub is_circle_hit: bool,
    pub is_inside_putt: bool,
    pub is_out_of_bounds: bool,
    pub is_outside_putt: bool,
    pub score: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct User {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Option<UserProfile>,
}
#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct UserProfile {
    pub profile_image_url: Option<String>,
    pub pdga_number: Option<f64>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "DNS")]
pub struct Dns {
    pub is_dns: bool,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "DNF")]
pub struct Dnf {
    pub is_dnf: bool,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Hole {
    pub par: Option<f64>,
    pub number: f64,
    pub length: Option<f64>,
    pub measure_in_meters: Option<bool>,
}

impl Default for Hole {
    fn default() -> Self {
        Self {
            par: None,
            number: 0.0,
            length: None,
            measure_in_meters: None,
        }
    }
}

pub mod round {
    use super::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct RoundsQueryVariables {
        pub event_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "RoundsQueryVariables")]
    pub struct RoundsQuery {
        #[arguments(eventId: $event_id)]
        pub event: Option<Event>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Event {
        pub rounds: Vec<Option<Round>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Round {
        pub id: cynic::Id,
    }
}

pub mod layout {
    pub use hole::*;

    use super::schema;

    pub mod hole {
        use std::sync::Arc;

        use itertools::Itertools;

        use crate::api::Error;

        use super::super::Division;

        #[derive(Debug, Clone, Default)]
        pub struct Holes {
            holes: Vec<Arc<Hole>>,
            division: Arc<Division>,
        }
        impl Holes {
            pub fn find_hole(&self, hole_number: u8) -> Option<Arc<Hole>> {
                self.holes
                    .iter()
                    .find(|h| h.hole == hole_number)
                    .map(Arc::clone)
            }

            pub fn from_vec_hole(holes: Vec<super::Hole>) -> Result<Self, Error> {
                let mut holes: Vec<Hole> = holes
                    .into_iter()
                    .map(|hole| Hole::try_from(hole))
                    .try_collect()?;
                holes.sort_by_key(|hole| hole.hole);
                let holes = holes.into_iter().map(Arc::new).collect();
                Ok(Self {
                    holes,
                    division: Division::default().into(),
                })
            }
        }

        #[derive(Debug, Clone, Default)]
        pub struct Hole {
            pub length: u16,
            pub par: u8,
            pub hole: u8,
        }

        impl TryFrom<super::Hole> for Hole {
            type Error = crate::api::Error;

            fn try_from(
                value: crate::controller::queries::layout::Hole,
            ) -> Result<Self, Self::Error> {
                let hole_number = value.number as u8;

                let length = if value.measure_in_meters.is_none() {
                    value.length
                } else if value.measure_in_meters.is_some_and(|s| s) {
                    value.length
                } else {
                    value.length.map(|l| l * 0.9144)
                };
                let length = length.ok_or(Self::Error::HoleLengthNotFound(hole_number))? as u16;
                let par = value.par.ok_or(Self::Error::HoleParNotFound(hole_number))? as u8;
                Ok(Hole {
                    length,
                    par,
                    hole: hole_number,
                })
            }
        }
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct HoleLayoutQueryVariables {
        pub event_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "HoleLayoutQueryVariables")]
    pub struct HoleLayoutQuery {
        #[arguments(eventId: $event_id)]
        pub event: Option<Event>,
    }
    #[derive(cynic::QueryFragment, Debug)]
    pub struct Event {
        pub rounds: Vec<Option<Round>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Round {
        pub pools: Vec<Pool>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Pool {
        pub layout_version: LayoutVersion,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct LayoutVersion {
        pub holes: Vec<Hole>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub(crate) struct Hole {
        pub measure_in_meters: Option<bool>,
        pub number: f64,
        pub name: Option<String>,
        pub par: Option<f64>,
        pub length: Option<f64>,
    }
}

// Groups
pub mod group {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use cynic::impl_scalar;
    use rocket_okapi::okapi::schemars;
    use schemars::JsonSchema;

    use crate::controller;

    use super::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct GroupsQueryVariables {
        pub event_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "GroupsQueryVariables")]
    pub struct GroupsQuery {
        #[arguments(eventId: $event_id)]
        pub event: Option<self::Event>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Event {
        pub rounds: Vec<Option<Round>>,
    }
    #[derive(cynic::QueryFragment, Debug)]
    pub struct Round {
        pub pools: Vec<Pool>,
    }
    #[derive(cynic::QueryFragment, Debug)]
    pub struct Pool {
        pub groups: Vec<Group>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct Group {
        pub id: cynic::Id,
        pub status: GroupStatus,
        pub position: f64,
        pub player_connections_v2: Vec<GroupPlayerConnectionTypeCombined>,
        pub start_hole: Option<super::Hole>,
        pub starts_at: Option<DateTime<Utc>>,
    }
    impl_scalar!(DateTime<Utc>, schema::DateTime);
    #[derive(cynic::InlineFragments, Debug, Clone)]
    pub enum GroupPlayerConnectionTypeCombined {
        GroupPlayerConnection(GroupPlayerConnection),
        #[cynic(fallback)]
        Unknown,
    }
    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct Player {
        pub id: cynic::Id,
        pub user: User,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct User {
        pub first_name: Option<String>,
        pub last_name: Option<String>,
    }
    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct GroupPlayerConnection {
        pub group_id: cynic::Id,
        pub player: Player,
    }
    #[derive(cynic::Enum, Clone, Copy, Debug, JsonSchema)]
    pub enum GroupStatus {
        Closed,
        Open,
        Done,
    }
}
#[cynic::schema("tjing")]
mod schema {}
