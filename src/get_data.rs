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

const DEFAULT_BG_COL: &'static str = "3F334D";

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

struct VmixInfo<'a> {
    id: &'a str,
    value: String,
    prop: VmixProperty,
}

enum VmixFunction<'a> {
    SetText(VmixInfo<'a>),
    SetPanX(VmixInfo<'a>),
    SetColor(VmixInfo<'a>),
    SetTextVisibleOn(VmixInfo<'a>),
    SetTextVisibleOff(VmixInfo<'a>),
    Restart(String),
    Play(String),
    OverlayInput4Off,
    OverlayInput4(VmixInfo<'a>)
}

impl VmixFunction<'_> {
    pub fn to_string(&self) -> String {
        match self {
            VmixFunction::SetText(info) => {
                match info.prop {
                    VmixProperty::Score(_, _) => {
                        if info.value == "0" {
                            return format!(
                                "FUNCTION SetText Value=E&Input={}&{}",
                                info.id,
                                info.prop.selection()
                            );
                        } else if info.value.parse::<i16>().unwrap_or(0) > 0 {
                            return format!(
                                "FUNCTION SetText Value=+{}&Input={}&{}",
                                info.value,
                                info.id,
                                info.prop.selection()
                            );
                        }
                    }
                    _ => {}
                }
                return format!(
                    "FUNCTION SetText Value={}&Input={}&{}",
                    info.value,
                    info.id,
                    info.prop.selection()
                );
            }
            VmixFunction::SetPanX(info) => format!(
                "FUNCTION SetPanX Value={}&Input={}&{}",
                info.value,
                info.id,
                info.prop.selection()
            ),
            VmixFunction::Restart(id) => format!("FUNCTION Restart Input={}", id),
            VmixFunction::Play(id) => format!("FUNCTION Play Input={}", id),
            VmixFunction::SetTextVisibleOn(info) => format!(
                "FUNCTION SetTextVisibleOn Input={}&{}",
                info.id,
                info.prop.selection()
            ),
            VmixFunction::SetTextVisibleOff(info) => format!(
                "FUNCTION SetTextVisibleOff Input={}&{}",
                info.id,
                info.prop.selection()
            ),
            VmixFunction::SetColor(info) => format!(
                "FUNCTION SetColor Value={}&Input={}&{}",
                info.value,
                info.id,
                info.prop.selection()
            ),
            VmixFunction::OverlayInput4Off => "FUNCTION OverlayInput4Off".to_string(),
            VmixFunction::OverlayInput4(info) => format!(
                "FUNCTION OverlayInput4 Input={}&{}",
                info.id,
                info.prop.selection()
            ),

        }
    }
}
#[derive(Clone)]
enum VmixProperty {
    Score(usize, usize),
    HoleNumber(usize, usize),
    Color(usize, usize),
    Name(usize),
    TotalScore(usize),
    Throw(usize),
    Mov(String),
}

impl VmixProperty {
    fn selection(&self) -> String {
        match self {
            VmixProperty::Score(v1, v2) => format!("SelectedName=s{}p{}.Text", v1, v2),
            VmixProperty::HoleNumber(v1, v2) => format!("SelectedName=HN{}p{}.Text", v1, v2),
            VmixProperty::Color(v1, v2) => format!("SelectedName=h{}p{}.Fill.Color", v1, v2),
            VmixProperty::Name(v1) => format!("SelectedName=namep{}.Text", v1),
            VmixProperty::TotalScore(v1) => format!("SelectedName=scoretotp{}.Text", v1),
            VmixProperty::Throw(ind) => format!("SelectedName=t#p{}.Text", ind),
            VmixProperty::Mov(id) => format!("SelectedName={}", id),
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
    pub total_score: i16, // Total score for all rounds
    round_score: i16, // Score for only current round
    round_ind: usize,
    rounds: Vec<PlayerRound>,
    div_id: cynic::Id,
    pub hole: usize,
    ind: usize,
    vmix_id: String,
    pub throws: u8,
    shift: usize,
    pub ob: bool,
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
            throws: 0,
            shift: 0,
            ob: false
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

    fn get_col(&self) -> String {
        self.rounds[self.round_ind].results[self.hole]
            .get_score_colour()
            .into()
    }

    // Below goes JS TCP Strings

    pub fn set_name(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.vmix_id,
            value: self.name.clone(),
            prop: VmixProperty::Name(self.ind),
        })
        .to_string()
        .into()
    }

    pub fn set_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        let result = self.rounds[self.round_ind].hole_score(self.hole);

        self.total_score =
            self.score_before_round() + self.rounds[self.round_ind].score_to_hole(self.hole);
        // Set score
        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: result.to_string(),
                prop: VmixProperty::Score(self.hole + 1 - self.shift, self.ind),
            })
            .to_string()
            .into(),
        );
        // Set colour
        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: self.get_col(),
                prop: VmixProperty::Color(self.hole + 1 - self.shift, self.ind),
            })
            .to_string()
            .into(),
        );
        // Show score
        return_vec.push(
            VmixFunction::SetTextVisibleOn(VmixInfo {
                id: &self.vmix_id,
                value: self.get_col(),
                prop: VmixProperty::Score(self.hole + 1 - self.shift, self.ind),
            })
            .to_string()
            .into(),
        );

        // HoleNumber
        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: self.hole.to_string(),
                prop: VmixProperty::HoleNumber(self.hole + 1, self.ind),
            })
            .to_string()
            .into(),
        );
        return_vec.push(
            VmixFunction::SetTextVisibleOn(VmixInfo {
                id: &self.vmix_id,
                value: self.get_col(),
                prop: VmixProperty::HoleNumber(self.hole + 1, self.ind),
            })
            .to_string()
            .into(),
        );

        return_vec.push(self.set_tot_score());
        self.hole += 1;
        self.throws = 0;
        return_vec.push(self.set_throw());

        return_vec
    }

    pub fn revert_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        if self.hole > 0 {
            self.hole -= 1;
            return_vec.append(&mut self.del_score());
            log(&format!("{}", self.hole));
            let result = self.rounds[self.round_ind].hole_score(self.hole);
            self.round_score -= result;
            self.total_score -= result;
            if self.hole > 8 {
                self.hole -= 1;
                self.round_score -= result;
                self.total_score -= result;
                return_vec.append(&mut self.shift_scores());
            } else {
                return_vec.push(self.set_tot_score());
            }
            
        }
        return_vec
    }

    fn set_tot_score(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.vmix_id,
            value: self.total_score.to_string(),
            prop: VmixProperty::TotalScore(self.ind),
        })
        .to_string()
        .into()
    }

    pub fn shift_scores(&mut self) -> Vec<JsString> {
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
        return_vec.append(&mut self.set_hole_score());
        return_vec
    }

    fn del_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        let score_prop = VmixProperty::Score(self.hole + 1, self.ind);
        let col_prop = VmixProperty::Color(self.hole + 1, self.ind);
        let h_num_prop = VmixProperty::HoleNumber(self.hole + 1, self.ind);
        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: "".to_string(),
                prop: score_prop.clone()
            })
            .to_string()
            .into(),
        );

        return_vec.push(
            VmixFunction::SetColor(VmixInfo {
                id: &self.vmix_id,
                value: format!("#{}", DEFAULT_BG_COL),
                prop: col_prop,
            })
            .to_string()
            .into(),
        );

        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: (self.hole + 1).to_string(),
                prop: h_num_prop,
            })
            .to_string()
            .into(),
        );
        return_vec.push(
            VmixFunction::SetTextVisibleOff(VmixInfo {
                id: &self.vmix_id,
                value: "".to_string(),
                prop: score_prop,
            })
            .to_string()
            .into(),
        );
        return_vec
    }

    pub fn reset_scores(&mut self) -> Vec<JsString> {
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

    pub fn set_throw(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.vmix_id,
            value: self.throws.to_string(),
            prop: VmixProperty::Throw(self.ind),
        })
        .to_string()
        .into()
    }

    pub fn start_score_anim(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        return_vec.push(VmixFunction::OverlayInput4Off.to_string().into());
        
        return_vec.push(self.set_input_pan());
        return_vec.push(VmixFunction::OverlayInput4(VmixInfo { id: &self.vmix_id, value: "".to_string(), prop: VmixProperty::Mov(self.get_mov()) }).to_string().into());
        self.ob = false;
        //return_vec.append(&mut self.play_anim());
        return_vec
    }

    fn set_input_pan(&mut self) -> JsString {
        let pan = match self.ind {
            1 => -0.628,
            2 => -0.628 + 0.419,
            3 => -0.628 + 0.4185 * 2.0,
            4 => -0.628 + 0.419 * 3.0,
            _ => -0.628,
        };
        VmixFunction::SetPanX(VmixInfo {
            id: &self.vmix_id,
            value: pan.to_string(),
            prop: VmixProperty::Mov(self.get_mov()),
        }).to_string().into()
    }

    fn play_anim(&mut self) -> Vec<JsString> {
        vec![
            VmixFunction::Restart(self.get_mov()).to_string().into(),
            VmixFunction::Play(self.get_mov()).to_string().into(),
        ]
    }

    fn get_mov(&self) -> String {
        if self.ob {
            "50 ob.mov".to_string()
        } else  {
            self.rounds[self.round_ind].results[self.hole].get_mov().to_string()
        } 
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
