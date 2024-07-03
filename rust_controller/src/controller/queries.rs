use crate::flipup_vmix_controls::Score;
use crate::vmix::functions::{VMixFunction, VMixProperty};
pub use group::{Group};

#[derive(cynic::QueryVariables, Debug)]
pub struct RoundResultsQueryVariables {
    pub event_id: cynic::Id,
    pub round_id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "RootQuery", variables = "RoundResultsQueryVariables")]
pub struct RoundResultsQuery {
    #[arguments(eventId: $event_id)]
    pub event: Option<Event>,
}

#[derive(cynic::QueryFragment, Debug,Clone)]
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

#[derive(cynic::QueryFragment, Debug,Clone)]
#[cynic(variables = "RoundResultsQueryVariables")]
pub struct Player {
    pub user: User,
    pub dnf: Dnf,
    pub dns: Dns,
    #[arguments(roundId: $round_id)]
    pub results: Option<Vec<HoleResult>>,
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
    pub player_connection: group::GroupPlayerConnection,
}





#[derive(cynic::QueryFragment, Debug,Clone)]
pub struct User {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub id: Option<cynic::Id>
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







impl From<&HoleResult> for Score {
    fn from(res: &HoleResult) -> Self {
        let par = res.hole.par.unwrap() as usize;
        Self::new(res.score as usize, par, res.hole.number as usize)
    }
}

impl HoleResult {
    pub fn actual_score(&self) -> f64 {
        if let Some(par) = self.hole.par {
            self.score - par
        } else {
            //log(&format!("no par for hole {}", self.hole.number));
            self.score
        }
    }

    fn to_score(&self) -> Score {
        self.try_into().unwrap()
    }

    pub fn get_score_colour(&self, player: usize) -> VMixFunction<VMixProperty> {
        self.to_score().update_score_colour(player)
    }

    pub fn get_mov(&self, player: usize) -> [VMixFunction<VMixProperty>; 3] {
        self.to_score().play_mov_vmix(player, false)
    }
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
    use cynic::{QueryFragment,QueryVariables};
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

// Groups
pub mod group {
    use rocket_okapi::okapi::schemars;
    use schemars::JsonSchema;
    use super::schema;

    #[derive(cynic::QueryFragment, Debug,Clone)]
    pub struct Group {
        pub id: cynic::Id,
        pub status: GroupStatus,
        pub position: f64,
        pub player_connections_v2: Vec<GroupPlayerConnectionTypeCombined>,
    }

    #[derive(cynic::InlineFragments, Debug,Clone)]
    pub enum GroupPlayerConnectionTypeCombined {
        GroupPlayerConnection(GroupPlayerConnection),
        #[cynic(fallback)]
        Unknown
    }
    #[derive(cynic::QueryFragment, Debug,Clone)]
    pub struct Player {
        pub id: cynic::Id,
        pub user: User,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct User {
        pub first_name: Option<String>,
        pub last_name: Option<String>,
    }
    #[derive(cynic::QueryFragment, Debug,Clone)]
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
