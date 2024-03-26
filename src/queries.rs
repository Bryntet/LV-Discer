use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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
pub struct Event {
    pub rounds: Vec<Option<Round>>,
    pub divisions: Vec<Option<Division>>,
}
#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Round {
    pub pools: Vec<Pool>,
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Division {
    pub id: cynic::Id,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct PoolLeaderboardDivision {
    pub id: cynic::Id,
    pub name: String,
    pub players: Vec<PoolLeaderboardPlayer>,
    #[cynic(rename = "type")]
    pub type_: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct PoolLeaderboardPlayer {
    pub first_name: String,
    pub is_dnf: bool,
    pub is_dns: bool,
    pub last_name: String,
    pub par: Option<f64>,
    pub pdga_number: Option<f64>,
    pub pdga_rating: Option<f64>,
    pub place: f64,
    pub player_id: cynic::Id,
    pub results: Vec<SimpleResult>,
    pub points: Option<f64>,
    pub score: Option<f64>, // Tror denna är total score för runda
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct SimpleResult {
    pub hole: Hole,
    pub score: f64,
    pub is_circle_hit: bool,
    pub is_inside_putt: bool,
    pub is_out_of_bounds: bool,
    pub is_outside_putt: bool,
}
impl SimpleResult {
    pub fn actual_score(&self) -> f64 {
        if let Some(par) = self.hole.par {
            self.score - par
        } else {
            //log(&format!("no par for hole {}", self.hole.number));
            self.score
        }
    }

    pub fn get_score_colour(&self) -> &str {
        match self.actual_score() as i64 {
            4 => "AB8E77FF", // TODO FIX CORRECT COLOUR
            3 => "AB8E77FF",
            2 => "CA988DFF",
            1 => "EC928FFF",
            0 => "7E8490FF",
            -1 => "A6F8BBFF",
            -2 => "6A8BE7FF",
            -3 => "DD6AC9FF",
            _ => "AB8E77FF",
        }
    }

    pub fn get_mov(&self) -> &str {
        match self.actual_score() as i64 {
            4 => "40 ouch.mov",
            3 => "30 3xBogey.mov",
            2 => "20 2xBogey.mov",
            1 => "10 bogey.mov",
            0 => "04 par.mov",
            -1 => "03 birdie.mov",
            -2 => "02 eagle.mov",
            -3 => {
                if self.score == 1.0 {
                    "00 ace.mov"
                } else {
                    "01 albatross.mov"
                }
            }
            _ => "",
        }
    }
}
#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Pool {
    pub status: PoolStatus,
    pub layout_version: LayoutVersion,
    pub leaderboard: Option<Vec<Option<PoolLeaderboardDivisionCombined>>>,
    pub position: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct LayoutVersion {
    pub holes: Vec<Hole>,
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

#[derive(cynic::InlineFragments, Debug, Clone)]
pub enum PoolLeaderboardDivisionCombined {
    Pld(PoolLeaderboardDivision),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum PoolStatus {
    Closed,
    Prepare,
    Open,
    Completed,
    Pause

}

#[cynic::schema("tjing")]
mod schema {}