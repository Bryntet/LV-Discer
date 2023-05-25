use cynic::GraphQlResponse;

use self::{
    queries::{Division, PoolLeaderboardDivision},
    schema::__fields::Pool::round,
};
use js_sys::JsString;
use wasm_bindgen::prelude::*;
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

const DEFAULT_BG_COL: String = "3F334D".to_string();

#[derive(Debug, Clone)]
pub struct PlayerRound {
    results: Vec<queries::SimpleResult>,
}
impl PlayerRound {
    fn new(results: Vec<queries::SimpleResult>) -> Self {
        Self { results }
    }

    // Gets score up until hole
    fn score_to_hole(&self, hole: usize) -> i16 {
        (0..hole + 1).map(|i| self.hole_score(i)).sum()
    }

    fn hole_score(&self, hole: usize) -> i16 {
        if let Some(par) = self.results[hole].hole.par {
            return self.results[hole].actual_score() as i16;
        }
        0
    }
}

struct VmixInfo {
    id: String,
    value: String,
    prop: VmixProperty,
}

enum VmixFunction {
    SetText(VmixInfo),
    SetPanX(VmixInfo),
    Restart(String),
    Play(String),
}

impl VmixFunction {
    pub fn to_string(&self) -> String {
        match self {
            VmixFunction::SetText(info) => format!("FUNCTION SetText Value={}&Input={}&{}", info.value, info.id, info.prop.selection()),
            VmixFunction::SetPanX(info) => format!("FUNCTION SetPanX Value={}&Input={}&{}", info.value, info.id, info.prop.selection()),
            VmixFunction::Restart(id) => format!("FUNCTION Restart Input={}", id),
            VmixFunction::Play(id) => format!("FUNCTION Play Input={}", id),
        }
    }
}

enum VmixProperty {
    Score(usize, usize),
    HoleNumber(usize, usize),
    Color(usize, usize),
    Name(usize)
}

impl VmixProperty {
    fn selection(&self) -> String {
        match self {
            VmixProperty::Score(v1, v2) => format!("SelectedName=s{}p{}.Text", v1, v2),
            VmixProperty::HoleNumber(v1, v2) => format!("SelectedName=HN{}p{}.Text", v1, v2),
            VmixProperty::Color(v1, v2) => format!("SelectedName=h{}p{}.Fill.Color", v1, v2),
            VmixProperty::Name(v1) => format!("SelectedName=namep{}.Text", v1),
        }
    }
}



#[derive(Debug, Clone)]
pub struct NewPlayer {
    pub player_id: cynic::Id,
    pub name: String,
    rank: queries::RankUpDown,
    best_score: bool,
    through: u8,
    total_score: i16, // Total score for all rounds
    round_score: i16, // Score for only current round
    round_ind: usize,
    rounds: Vec<PlayerRound>,
    div_id: cynic::Id,
    hole: usize,
    ind: usize,
    vmix_id: String,
}

impl Default for NewPlayer {
    fn default() -> Self {
        Self {
            player_id: cynic::Id::from(""),
            name: "".to_string(),
            rank: queries::RankUpDown::Same,
            best_score: false,
            through: 0,
            total_score: 0,
            round_score: 0,
            round_ind: 0,
            rounds: vec![],
            div_id: cynic::Id::from(""),
            hole: 0,
            ind: 0,
            vmix_id: "".to_string(),
        }
    }
}

impl NewPlayer {
    pub fn new(
        id: cynic::Id,
        f_name: String,
        l_name: String,
        event: queries::Event,
        div_id: cynic::Id,
    ) -> Self {
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
                        }
                        Some(queries::PoolLeaderboardDivisionCombined::Unknown) => {}
                        None => {}
                    }
                }
            }
        }

        Self {
            player_id: id,
            name: format!("{} {}", f_name, l_name),
            best_score: false,
            rounds,
            div_id,
            ..Default::default()
        }
    }

    pub fn get_round_total_score(&self, round_ind: usize) -> i16 {
        self.rounds[round_ind].score_to_hole(17)
    }

    fn score_before_round(&self) -> i16 {
        let mut total_score = 0;
        for round_ind in 0..self.round_ind {
            total_score += self.get_round_total_score(round_ind)
        }
        total_score
    }

    fn generate_identifier(&self, s_1: &str, s_2: &str) -> String {
        format!("{}{}{}{}", s_1, self.hole + 1, s_2, self.ind)
    }

    // Below goes JS TCP Strings

    pub fn set_name(&mut self) -> JsString {
        let inner_name = format!("namep{}", self.ind);
        let selection = format!("Input={}&SelectedName={}.Text", self.vmix_id, inner_name,);
        format!("FUNCTION SetText Value={}&{}", self.name, &selection).into()
    }

    fn set_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        // self.start_score_anim();
        // wait Xms
        let selection = format!(
            "Input={}&{}",
            self.vmix_id,
            VmixProperty::Score.selection(self.hole+1, self.ind)
        );
        let select_colour = format!(
            "Input={}&{}",
            self.vmix_id,
            VmixProperty::Color.selection(self.hole+1, self.ind)
        );
        let selection_hole = format!(
            "Input={}&{}",
            self.vmix_id,
            VmixProperty::HoleNumber.selection(self.hole+1, self.ind)
        );
        let result = &player.results[self.hole];

        // Set score
        return_vec
            .push(format!("FUNCTION SetText Value={}&{}", &result.score, &selection).into());
        // Set colour
        return_vec.push(
            format!(
                "FUNCTION SetColor Value=#{}&{}",
                &result.get_score_colour(),
                &select_colour
            )
            .into(),
        );
        // Show score
        return_vec.push(format!("FUNCTION SetTextVisibleOn {}", &selection).into());
        return_vec.push(format!("FUNCTION SetTextVisibleOn {}", &selection_hole).into());
        return_vec.push(
            format!(
                "FUNCTION SetText Value={}&{}",
                &self.hole + 1,
                &selection_hole
            )
            .into(),
        );

        self.score += result.actual_score();
        return_vec.push(self.set_tot_score());
        self.hole += 1;
        self.throws = 0;
        return_vec.push(self.set_throw());
        
        return_vec
    }

    fn set_tot_score(&self) -> JsString {
        let n = format!("scoretotp{}", self.ind);
        let selection = format!("Input={}&SelectedName={}.Text", self.vmix_id, n);
        format!("FUNCTION SetText Value={}&{}", self.total_score, &selection).into()
    }

    fn shift_scores(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        let in_hole = self.hole.clone();
        let diff = self.hole - 8;
        self.hole = diff;
        log(&format!("diff: {}", diff));
        for i in diff..in_hole {
            log(&format!("i: {}", i));
            self.shift = diff;
            log(&format!("hole: {}\nshift: {}", self.hole, self.shift));
            return_vec.append(&mut self.set_hole_score());
        }
        self.rounds[self.round_ind].self.score = score;
        return_vec.append(&mut self.set_hole_score());
        return_vec
    }

    fn del_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        let selection = format!(
            "Input={}&SelectedName={}.Text",
            self.vmix_id,
            self.generate_identifier("s", "p")
        );
        let select_colour = format!(
            "Input={}&SelectedName={}.Fill.Color",
            self.vmix_id,
            self.generate_identifier("h", "p")
        );
        let selection_hole = format!(
            "Input={}&SelectedName={}.Text",
            self.vmix_id,
            self.generate_identifier("HN", "p")
        );
        return_vec.push(format!("FUNCTION SetText Value={}&{}", "", &selection).into());
        return_vec.push(
            format!(
                "FUNCTION SetColor Value=#{}&{}",
                DEFAULT_BG_COL, &select_colour
            )
            .into(),
        );
        return_vec.push(
            format!(
                "FUNCTION SetText Value={}&{}",
                &self.hole + 1,
                &selection_hole
            )
            .into(),
        );
        return_vec.push(format!("FUNCTION SetTextVisibleOff {}", &selection).into());
        return_vec
    }

    fn reset_scores(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        for i in 0..9 {
            self.hole = i;
            return_vec.append(&mut self.del_score());
        }
        self.hole = 0;
        self.round_score = 0;
        return_vec.push(self.set_tot_score());
        return_vec
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
            for div in &self.event.rounds[self.round_ind]
                .clone()
                .expect("no round")
                .pools[*ind]
                .clone()
                .leaderboard
                .expect("no leaderboard")
            {
                match div {
                    Some(queries::PoolLeaderboardDivisionCombined::PLD(division)) => {
                        if division.id == self.chosen_division {
                            for player in &division.players {
                                players.push(player.clone());
                            }
                        }
                    }
                    Some(queries::PoolLeaderboardDivisionCombined::Unknown) => {}
                    None => {}
                }
            }
        }
        for player in players {
            out_vec.push(NewPlayer::new(
                player.player_id,
                player.first_name,
                player.last_name,
                self.event.clone(),
                self.chosen_division.clone(),
            ));
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
    #[derive(Debug, Clone)]
    pub enum RankUpDown {
        Up(i8),
        Down(i8),
        Same,
    }
    impl Default for RankUpDown {
        fn default() -> Self {
            RankUpDown::Same
        }
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
