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

enum ReadableScore {
    Bogey(BogeyType),
    Par,
    Birdie,
    Eagle,
    Albatross,
    Ace,
}

enum BogeyType {
    Single,
    Double,
    Triple,
    Ouch
}
impl BogeyType {
    const fn new(score: u8) -> Self {
        match score {
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Triple,
            _ => Self::Ouch
        }
    }
}
impl From<&SimpleResult> for ReadableScore {
    fn from(res: &SimpleResult) -> Self {
        Self::new(res.score as i8,res.hole.par.map(|p|p as i8))
    }
}

impl ReadableScore {
    const fn new(score: i8, par: Option<i8>) -> Self {
        let actual_score = if let Some(par) = par {
            score - par
        } else {
            score
        };
        match actual_score {
            0 => Self::Par,
            -1 => Self::Birdie,
            -2 => Self::Eagle,
            -3 if score == 1 => Self::Ace,
            -3 => Self::Albatross,
            ..=-3 => Self::Ace,
            1.. => Self::Bogey(BogeyType::new(actual_score as u8))
        }
    } 
    
    const fn to_colour(&self) -> &'static str {
        use ReadableScore::*;
        match self {
            Bogey(bogey_type) => {
                match bogey_type {
                    BogeyType::Triple | BogeyType::Ouch => "AB8E77FF",
                    BogeyType::Double => "CA988DFF",
                    BogeyType::Single => "EC928FFF",
                }
            },
            Par => "7E8490FF",
            Birdie => "A6F8BBFF",
            Eagle => "6A8BE7FF",
            Ace | Albatross => "DD6AC9FF"
        }
    }
    
    const fn to_mov(&self) -> &'static str {
        use ReadableScore::*;
        match self {
            Bogey(bogey_type) => match bogey_type {
                BogeyType::Ouch => "40 ouch.mov",
                BogeyType::Triple => "30 3xBogey.mov",
                BogeyType::Double => "20 2xBogey.mov",
                BogeyType::Single => "10 bogey.mov"
            },
            Par => "04 par.mov",
            Birdie => "03 birdie.mov",
            Eagle => "02 eagle.mov",
            Albatross => "01 albatross.mov",
            Ace => "00 ace.mov"
        }
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
    
    fn readable_score(&self) -> ReadableScore {
        self.into()
    }
    
    pub fn get_score_colour(&self) -> &'static str {
        self.readable_score().to_colour()
    }

    pub fn get_mov(&self) -> &'static str {
        self.readable_score().to_mov()
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