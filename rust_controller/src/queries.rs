use crate::flipup_vmix_controls::Score;
use crate::vmix::functions::{VMixFunction, VMixProperty, VMixSelectionTrait};
use js_sys::JsString;
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

impl From<&SimpleResult> for Score {
    fn from(res: &SimpleResult) -> Self {
        let par = res.hole.par.unwrap() as usize;
        Self::new(res.score as usize, par, res.hole.number as usize)
    }
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
    Pause,
}

#[cynic::schema("tjing")]
mod schema {}
