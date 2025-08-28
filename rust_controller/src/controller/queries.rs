pub use group::Group;
use rocket_okapi::JsonSchema;
use serde::Serialize;

pub mod results_getter {
    use super::schema;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(cynic::QueryVariables, Debug)]
    struct RoundResultsQueryVariables {
        pub round_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "RootQuery", variables = "RoundResultsQueryVariables")]
    struct RoundResultsQuery {
        #[arguments(roundId: $round_id)]
        pub round: Option<Round>,
    }

    #[derive(Deserialize)]
    struct RoundResultsGetter {
        pub data: RoundResultsQuery,
    }
    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "Round")]
    struct Round {
        pub pools: Vec<Pool>,
    }
    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "Pool")]
    struct Pool {
        pub groups: Vec<Group>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "Group")]
    struct Group {
        pub results: Vec<InternalHoleResult>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "Result")]
    struct InternalHoleResult {
        score: f64,
        hole: HoleNumber,
        is_circle_hit: bool,
        is_inside_putt: bool,
        is_out_of_bounds: bool,
        is_outside_putt: bool,
        is_verified: bool,
        player_connection: GroupPlayerConnection,
    }
    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "Hole")]
    struct HoleNumber {
        pub number: f64,
        pub par: Option<f64>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "GroupPlayerConnection")]
    struct GroupPlayerConnection {
        pub player_id: cynic::Id,
    }

    pub struct PlayerResult {
        pub player_id: cynic::Id,
        pub results: Vec<HoleResult>,
    }

    #[derive(Debug)]
    pub struct PlayerResults(pub HashMap<cynic::Id, Vec<HoleResult>>);

    #[derive(Debug, Clone)]
    pub struct HoleResult {
        pub score: usize,
        pub hole_number: usize,
        pub par: u8,
        pub is_circle_hit: bool,
        pub is_inside_putt: bool,
        pub is_out_of_bounds: bool,
        pub is_outside_putt: bool,
        pub is_verified: bool,
    }

    pub async fn get_round_results(round_id: cynic::Id) -> Option<PlayerResults> {
        use cynic::QueryBuilder;
        use itertools::Itertools;
        let query = RoundResultsQuery::build(RoundResultsQueryVariables {
            round_id: round_id.clone(),
        });
        let Ok(response) = reqwest::Client::new()
            .post("https://api.tjing.se/graphql")
            .json(&query)
            .send()
            .await
        else {
            return None;
        };

        let res = response.bytes().await;

        let test =
            serde_json::from_slice::<RoundResultsGetter>(&res.unwrap()).map(|result| result.data);

        match test {
            Ok(RoundResultsQuery {
                round: Some(Round { pools }),
            }) => {
                let results = pools
                    .into_iter()
                    .flat_map(|pool| pool.groups)
                    .flat_map(|group| group.results)
                    .collect_vec();
                let mut player_map = HashMap::new();

                for result in results {
                    let player_id = result.player_connection.player_id.clone();
                    let hole_number = result.hole.number as usize;
                    let score = result.score as usize;

                    let Some(par) = result.hole.par.map(|par| par as u8) else {
                        println!("BIG ISSUE, par not found");
                        continue;
                    };

                    let hole_result = HoleResult {
                        score,
                        hole_number,
                        par,
                        is_circle_hit: result.is_circle_hit,
                        is_inside_putt: result.is_inside_putt,
                        is_out_of_bounds: result.is_out_of_bounds,
                        is_outside_putt: result.is_outside_putt,
                        is_verified: result.is_verified,
                    };

                    player_map
                        .entry(player_id)
                        .or_insert_with(Vec::new)
                        .push(hole_result);
                }
                Some(PlayerResults(player_map))
            }
            Ok(RoundResultsQuery { round: None }) => {
                eprintln!(
                    "No results found for round with ID: {}",
                    round_id.into_inner()
                );
                None
            }
            Err(e) => {
                eprintln!("Error parsing response: {}", e);
                None
            }
        }
    }
}

#[derive(cynic::QueryVariables, Debug)]
pub struct EventQueryVariables {
    pub event_id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "RootQuery", variables = "EventQueryVariables")]
pub struct EventQuery {
    #[arguments(eventId: $event_id)]
    pub event: Option<Event>,
}
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Event")]
pub struct Event {
    pub divisions: Vec<Option<Division>>,
    pub rounds: Vec<Option<Round>>,
    pub players: Vec<Player>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Round {
    pub id: cynic::Id,
}
#[derive(cynic::QueryFragment, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Division {
    pub name: String,
    #[cynic(rename = "type")]
    #[serde(rename = "type")]
    pub short_name: String,
    pub id: cynic::Id,
}

impl Default for Division {
    fn default() -> Self {
        Division {
            name: "".to_string(),
            short_name: "".to_string(),
            id: cynic::Id::new(""),
        }
    }
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Player {
    pub user: User,
    pub dnf: Dnf,
    pub dns: Dns,
    pub division: Division,
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
    pub is_verified: bool,
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
        #[arguments(eventId: $ event_id)]
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

        #[derive(Debug, Clone, Default)]
        pub struct Holes {
            holes: Vec<Arc<Hole>>,
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
                Ok(Self { holes })
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
                let length = length.unwrap_or(0.) as u16;
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
        #[arguments(eventId: $ event_id)]
        pub event: Option<Event>,
    }
    #[derive(cynic::QueryFragment, Debug)]
    pub struct Event {
        pub rounds: Vec<Option<Round>>,
        pub division_in_pool: Vec<Option<DivisionInPoolType>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct DivisionInPoolType {
        pub division_id: cynic::Id,
        pub pool_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Round {
        pub pools: Vec<Pool>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct Pool {
        pub layout_version: Option<LayoutVersion>,
        pub id: cynic::Id,
        pub groups: Vec<Group>,
        pub name: Option<String>,
    }
    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct Group {
        pub id: cynic::Id,
    }
    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct LayoutVersion {
        pub holes: Vec<Hole>,
        pub layout: Layout,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct Layout {
        pub name: String,
        pub course: Option<Course>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct Course {
        pub name: String,
    }

    #[derive(cynic::QueryFragment, Debug, Default, Clone)]
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

    use super::schema;
    use crate::controller;
    use crate::controller::queries::layout::LayoutVersion;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct GroupsQueryVariables {
        pub event_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "GroupsQueryVariables")]
    pub struct GroupsQuery {
        #[arguments(eventId: $ event_id)]
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
    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct Pool {
        pub groups: Vec<Group>,
        pub layout_version: Option<LayoutVersion>,
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
