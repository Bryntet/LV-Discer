use cynic::GraphQlResponse;

use wasm_bindgen::prelude::*;

use self::{queries::{PoolLeaderboardDivision, Division}, schema::__fields::Pool::round};
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub async fn post_status(event_id: cynic::Id) -> cynic::GraphQlResponse<queries::EventQuery> {
    use cynic::QueryBuilder;
    use queries::*;
    let operation = EventQuery::build(EventQueryVariables {
        event_id: event_id.clone(),
    });
    println!("operation: {:#?}", event_id);

    let response = reqwest::Client::new()
        .post("https://api.tjing.se/graphql")
        .json(&operation)
        .send()
        .await
        .expect("failed to send request");
    response
        .json::<GraphQlResponse<queries::EventQuery>>()
        .await
        .expect("failed to parse response")
}
#[derive(Debug)]
pub struct PlayerRound {
    results: Vec<queries::SimpleResult>,
}
impl PlayerRound {
    fn new(results: Vec<queries::SimpleResult>) -> Self {
        Self {
            results
        }
    }

    fn get_score(&self, hole: usize) -> i16 {
        for i in 0..hole+1 {
            if let Some(par) = self.results[i].hole.par {
                return self.results[i].actual_score() as i16;
            }
        }
        0
    }
    
}
#[derive(Debug)]
pub struct NewPlayer {
    pub player_id: cynic::Id,
    pub name: String,
    rank: queries::RankUpDown,
    best_score: bool,
    through: u8,
    current_score: i16,
    round_score: i16,
    round_ind: usize,
    rounds: Vec<PlayerRound>,
    div_id: cynic::Id,
    hole: usize
}

impl NewPlayer {
    pub fn new(id: cynic::Id, f_name: String, l_name: String, event: queries::Event, div_id: cynic::Id, ) -> Self {
        let mut rounds: Vec<PlayerRound> = vec![];
        for rnd in event.rounds {
            for pool in rnd.expect("no round").pools {
                for player in pool.leaderboard.expect("no lb") {
                    match player {
                        Some(queries::PoolLeaderboardDivisionCombined::PLD(division)) => {
                            if division.id == div_id {
                                for player in division.players {
                                    if player.player_id == id {
                                        rounds.push(PlayerRound::new(player.results));
                                    }
                                }
                            }
                        },
                        Some(queries::PoolLeaderboardDivisionCombined::Unknown) => {},
                        None => {},
                    }
                }
            }
        }

        Self {
            player_id: id,
            name: format!("{} {}", f_name, l_name),
            rank: queries::RankUpDown::Same,
            best_score: false,
            through: 0,
            current_score: 0,
            round_score: 0,
            round_ind: 0,
            rounds,
            div_id,
            hole: 0,
        }
    }

    pub fn get_round_total_score(&self, round_ind: usize) -> i16 {
        self.rounds[round_ind].get_score(17)
    }

    fn score_before_round(&self) -> i16 {
        let mut total_score = 0;
        for round_ind in 0..self.round_ind {
            total_score += self.get_round_total_score(round_ind)
        }
        total_score
    }
}

#[derive(Clone)]
pub struct RustHandler {
    pub chosen_division: cynic::Id,
    event: queries::Event,
    divisions: Vec<queries::Division>,
    round_id: cynic::Id,
    round_ind: usize,
    valid_pool_inds: Vec<usize>,
}

impl RustHandler {
    pub fn new(pre_event: GraphQlResponse<queries::EventQuery>) -> Self {

        let event = pre_event.data.expect("no data").event.expect("no event");
        let mut divisions: Vec<queries::Division> = vec![];
        for div in &event.divisions {
            if let Some(div) = div {
                divisions.push(div.clone());
            }
        }
        
        Self {
            chosen_division: divisions[0].id.clone(),
            event,
            divisions,
            round_id: cynic::Id::from(""),
            round_ind: 0,
            valid_pool_inds: vec![0],
        }
    }
    
    pub fn get_divisions(&self) -> Vec<Division> {
        let mut divs: Vec<Division> = vec![];
        for div in &self.divisions {
            divs.push(div.clone());
        }
        divs
    }

    pub fn get_round(&self) -> queries::Round {
        self.event.rounds[self.round_ind].clone().expect("no round")
    }

    pub fn get_players(&self) -> Vec<NewPlayer> {
        let mut players: Vec<queries::PoolLeaderboardPlayer> = vec![];
        let mut out_vec: Vec<NewPlayer> = vec![];
        for ind in &self.valid_pool_inds {
            for div in &self.event.rounds[self.round_ind].clone().expect("no round").pools[*ind].clone().leaderboard.expect("no leaderboard") {
                match div {
                    Some(queries::PoolLeaderboardDivisionCombined::PLD(division)) => {
                        if division.id == self.chosen_division {
                            for player in &division.players {
                                players.push(player.clone());
                            }
                        }
                    },
                    Some(queries::PoolLeaderboardDivisionCombined::Unknown) => {},
                    None => {},
                }
            }
        }
        for player in players {
            out_vec.push(NewPlayer::new(player.player_id, player.first_name, player.last_name, self.event.clone(), self.chosen_division.clone()));
        }
        out_vec
    }
    pub fn set_chosen_by_ind(&mut self, ind: usize) {
        self.chosen_division = self.divisions[ind].id.clone();
    }
}

#[cynic::schema_for_derives(file = r#"src/schema.graphql"#, module = "schema")]
pub mod queries {
    use std::default;

    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }

    use super::schema;

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
    #[derive(Debug)]
    pub enum RankUpDown {
        Up(i8),
        Down(i8),
        Same,
    }
    impl Default for RankUpDown {
        fn default() -> Self { RankUpDown::Same }
    }


    #[derive(Default)]
    struct LBInformation {
        position: u8,
        round_score: i16,
        total_score: i16,
        player_name: String,
        rank: RankUpDown,
        best_score: bool,
        through: u8,
    }

    impl PoolLeaderboardPlayer {

        fn get_current_round(&self, through: u8) -> LBInformation {
            LBInformation::default()
        }

        fn total_score(&self) -> i16 {
            let mut total_score = 0;
            for result in &self.results {
                total_score += result.actual_score() as i16;
            }
            total_score
        }

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
                log(&format!("no par for hole {}", self.hole.number));
                self.score
            }
        }

        pub fn get_score_colour(&self) -> &str {
            match self.actual_score() as i64 {
                4 => "AB8E77", // TODO FIX CORRECT COLOUR
                3 => "AB8E77",
                2 => "CA988D",
                1 => "EC928F",
                0 => "7E8490",
                -1 => "A6F8BB",
                -2 => "6A8BE7",
                -3 => "DD6AC9",
                _ => "AB8E77",
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
    }


    #[derive(cynic::InlineFragments, Debug, Clone)]
    pub enum PoolLeaderboardDivisionCombined {
        PLD(PoolLeaderboardDivision),
        #[cynic(fallback)]
        Unknown,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    pub enum PoolStatus {
        Closed,
        Prepare,
        Open,
        Completed,
    }

    
}
#[allow(non_snake_case, non_camel_case_types)]
mod schema {
    cynic::use_schema!(r#"src/schema.graphql"#);
}
