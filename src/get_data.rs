use self::queries::Division;
use cynic::GraphQlResponse;
use js_sys::JsString;
use wasm_bindgen::prelude::*;
use crate::vmix_controller::*;

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

const DEFAULT_FOREGROUND_COL: &str = "3F334D";
const DEFAULT_BACKGROUND_COL: &str = "574B60";
#[derive(Debug, Clone, Default)]
pub enum RankUpDown {
    Up(i16),
    Down(i16),
    #[default]
    Same,
}

pub trait HoleScoreOrDefault {
    fn hole_score(&self, hole: usize) -> i16;
    fn score_to_hole(&self, hole: usize) -> i16;
    fn get_score_colour(&self, hole: usize) -> String;
    fn get_hole_info(&self, hole: usize) -> Vec<JsString>;
}

// Implement the trait for `Option<&PlayerRound>`
impl HoleScoreOrDefault for Option<&PlayerRound> {
    fn hole_score(&self, hole: usize) -> i16 {
        match self {
            Some(round) => round.hole_score(hole),
            None => i16::MAX,
        }
    }
    fn score_to_hole(&self, hole: usize) -> i16 {
        match self {
            Some(round) => round.score_to_hole(hole),
            None => i16::MAX,
        }
    }
    fn get_hole_info(&self, hole: usize) -> Vec<JsString> {
        match self {
            Some(round) => round.get_hole_info(hole),
            None => vec![],
        }
    }
    fn get_score_colour(&self, hole: usize) -> String {
        match self {
            Some(round) => match round.results.get(hole) {
                Some(result) => result.get_score_colour().into(),
                None => "000000".to_string(),
            },
            None => "000000".to_string(),
        }
    }
}

impl RankUpDown {
    fn get_tcps(&self, pos: u16,) -> Vec<JsString> {
        let the_vec = vec![
            self.make_move(pos).into(),
            self.make_arrow(pos).into(),
        ];
        //log(&format!("{:#?}", the_vec));
        the_vec
    }

    fn make_move(&self, pos: u16) -> String {
        let movement = match self {
            RankUpDown::Up(val) => val.to_string(),
            RankUpDown::Down(val) => val.to_string(),
            RankUpDown::Same => "".to_string(),
        };

        VMixFunction::SetText{
            value: movement,
            input: LeaderBoardProperty::Move{pos}.into(),
        }
            .to_cmd()
    }

    fn make_arrow(&self, pos: u16) -> String {
        let img = match self {
            RankUpDown::Up(_) => r"x:\FLIPUP\grafik\greentri.png",
            RankUpDown::Down(_) => r"x:\FLIPUP\grafik\redtri.png",
            RankUpDown::Same => r"x:\FLIPUP\grafik\alpha.png",
        }.to_string();

        VMixFunction::SetImage{
            value: img,
            input: LeaderBoardProperty::Arrow{pos}.into(),
        }.to_cmd()
    }
}

#[derive(Debug, Clone)]
pub struct PlayerRound {
    results: Vec<queries::SimpleResult>,
}
impl PlayerRound {
    fn new(results: Vec<queries::SimpleResult>) -> Self {
        Self { results }
    }

    // Gets score up until hole
    pub fn score_to_hole(&self, hole: usize) -> i16 {
        //log(&format!("hole {}", hole));
        (0..hole + 1).map(|i| self.hole_score(i)).sum()
    }

    fn hole_score(&self, hole: usize) -> i16 {
        match self.results.get(hole) {
            Some(result) => result.actual_score() as i16,
            None => i16::MAX,
        }
    }

    pub fn get_hole_info(&self, hole: usize) -> Vec<JsString> {
        let id = "0e76d38f-6e8d-4f7d-b1a6-e76f695f2094";

        let mut r_vec: Vec<JsString> = vec![];
        let binding = self::queries::Hole::default();
        let hole = match &self.results.get(hole) {
            Some(hole) => &hole.hole,
            None => &binding,
        };
        r_vec.push(
            VMixFunction::SetText{
                value: hole.number.to_string(),
                input: VmixProperty::Hole.into(),
            }
                .to_cmd()
                .into(),
        );

        r_vec.push(
            VMixFunction::SetText{
                value: hole.par.expect("Par should always be set").to_string(),
                input: VmixProperty::HolePar.into(),
            }
                .to_cmd()
                .into(),
        );
        let meters = if hole.measure_in_meters.unwrap_or(false) {
            hole.length.unwrap_or(0.0)
        } else {
            hole.length.unwrap_or(0.0) * 0.9144
        };

        r_vec.push(
            VMixFunction::SetText{
                value: (meters as u64).to_string() + "M",
                input: VmixProperty::HoleMeters.into(),
            }
                .to_cmd()
                .into(),
        );

        let feet = (meters * 3.28084) as u64;
        r_vec.push(
            VMixFunction::SetText{
                value: feet.to_string() + "FT",
                input: VmixProperty::HoleFeet.into(),
            }
                .to_cmd()
                .into(),
        );
        r_vec
    }
}



fn fix_score(score: i16) -> String {
    use std::cmp::Ordering;

    match score.cmp(&0) {
        Ordering::Less => format!("{}", score),
        Ordering::Equal => "E".to_string(),
        Ordering::Greater => format!("%2B{}", score),
    }
}



#[derive(Debug, Clone)]
pub struct NewPlayer {
    pub player_id: cynic::Id,
    pub name: String,
    first_name: String,
    surname: String,
    pub rank: RankUpDown,
    pub total_score: i16, // Total score for all rounds
    pub round_score: i16, // Score for only current round
    round_ind: usize,
    pub rounds: Vec<PlayerRound>,
    pub hole: usize,
    pub ind: usize,
    vmix_id: String,
    pub throws: u8,
    shift: usize,
    pub ob: bool,
    pub position: u16,
    pub lb_even: bool,
    pub hot_round: bool,
    pub lb_vmix_id: String,
    pub lb_pos: u16,
    pub old_pos: u16,
    pos_visible: bool,
    pub lb_shown: bool,
    pub dnf: bool,
    pub first_scored: bool,
    pub thru: u8,
    pub visible_player: bool,
}

impl Default for NewPlayer {
    fn default() -> Self {
        Self {
            player_id: cynic::Id::from(""),
            name: "".to_string(),
            first_name: "".to_string(),
            surname: "".to_string(),
            rank: RankUpDown::Same,
            total_score: 0,
            round_score: 0,
            round_ind: 0,
            rounds: vec![],
            hole: 0,
            ind: 0,
            vmix_id: "".to_string(),
            throws: 0,
            shift: 0,
            ob: false,
            position: 0,
            lb_even: false,
            hot_round: false,
            lb_vmix_id: "".to_string(),
            lb_pos: 0,
            old_pos: 0,
            pos_visible: true,
            lb_shown: true,
            dnf: false,
            first_scored: false,
            thru: 0,
            visible_player: true,
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
        vmix_id: String,
        lb_vmix_id: String,
    ) -> Self {
        let mut rounds: Vec<PlayerRound> = vec![];
        for rnd in event.rounds {
            for pool in rnd.expect("no round").pools {
                for player in pool.leaderboard.expect("no lb") {
                    match player {
                        Some(queries::PoolLeaderboardDivisionCombined::Pld(division)) => {
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
            first_name: f_name,
            surname: l_name,
            rounds,
            vmix_id,
            lb_vmix_id,
            ..Default::default()
        }
    }

    pub fn get_round_total_score(&self) -> i16 {
        self.current_round().score_to_hole(17)
    }

    pub fn score_before_round(&mut self) -> i16 {
        let mut total_score = 0;
        if self.rounds.len() < self.round_ind || self.dnf {
            println!("I'm a loser, I DNF:ed");
            self.dnf = true;
        } else {
            for round_ind in 0..self.round_ind {
                total_score += self.rounds[round_ind].score_to_hole(17)
            }
        }

        // log(&format!("round_ind {} tot_score {}", self.round_ind, total_score));
        total_score
    }

    fn get_col(&self) -> String {
        self.current_round().get_score_colour(self.hole)
    }

    pub fn current_round(&self) -> Option<&PlayerRound> {
        self.rounds.get(self.round_ind)
    }
    pub fn check_if_allowed_to_visible(&mut self) {
        if self.round_ind >= self.rounds.len() {
            self.lb_shown = false;
        }
    }

    pub fn make_tot_score(&mut self) {
        self.round_score = self.current_round().score_to_hole(self.hole);
        // log(&format!(
        //     "round_score {} hole {}",
        //     self.round_score, self.hole
        // ));
        self.total_score = self.score_before_round() + self.round_score;
        //log(&format!("total_score {}", self.total_score));
    }

    pub fn check_pos(&mut self) {
        if self.old_pos < self.lb_pos && self.round_ind != 0 {
            self.rank = RankUpDown::Down((self.old_pos as i16 - self.lb_pos as i16).abs());
        } else if self.old_pos > self.lb_pos && self.round_ind != 0 {
            self.rank = RankUpDown::Up((self.lb_pos as i16 - self.old_pos as i16).abs());
        } else {
            self.rank = RankUpDown::Same;
        }
    }

    // Below goes JS TCP Strings

    pub fn set_name(&self) -> Vec<JsString> {
        vec![
            VMixFunction::SetText{
                value: self.first_name.clone(),
                input: VmixProperty::Name(self.ind).into(),
            }
                .to_cmd()
                .into(),
            VMixFunction::SetText{
                value: self.surname.clone(),
                input: VmixProperty::Surname(self.ind).into(),
            }
                .to_cmd()
                .into(),
        ]
    }

    pub fn set_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        // log(&format!("{}", self.round_ind));
        // log(&format!("{:#?}", self.rounds));
        let result = self.current_round().hole_score(self.hole);
        if !self.first_scored {
            self.first_scored = true;
        }
        self.make_tot_score();
        // Set score
        let pos = self.hole + 1 - self.shift;

        return_vec.push(
            VMixFunction::SetText{
                value: result.to_string(),
                input: VmixProperty::Score(pos, self.ind).into(),
            }
                .to_cmd()
                .into(),
        );

        // Set colour
        return_vec.push(
            VMixFunction::SetColor{
                color: self.get_col(),
                input: VmixProperty::ScoreColor(pos, self.ind).into(),
            }
                .to_cmd()
                .into(),
        );
        // Show score
        return_vec.push(
            VMixFunction::SetTextVisibleOn {
                input: VmixProperty::Score(pos, self.ind).into(),
            }
                .to_cmd()
                .into(),
        );

        // HoleNumber
        return_vec.push(
            VMixFunction::SetText {
                value: (self.hole + 1).to_string(),
                input: VmixProperty::HoleNumber(pos, self.ind).into(),
            }
                .to_cmd()
                .into(),
        );
        // return_vec.push(
        //     VmixFunction::SetTextVisibleOn(VmixInfo {
        //         id: &self.vmix_id,
        //         value: self.get_col(),
        //         prop: VmixProperty::HoleNumber(self.hole + 1, self.ind),
        //     })
        //     .to_string()
        //     .into(),
        // );

        return_vec.push(self.set_tot_score());
        if self.round_ind > 0 {
            return_vec.append(&mut self.set_round_score());
        }
        self.hole += 1;
        self.throws = 0;
        return_vec.push(self.set_throw());
        if self.visible_player {
            return_vec
        } else {
            vec![]
        }
    }

    pub fn revert_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        if self.hole > 0 {
            self.hole -= 1;
            return_vec.append(&mut self.del_score());
            let result = self.current_round().hole_score(self.hole);
            self.round_score -= result;
            self.total_score -= result;
            if self.hole > 8 {
                self.hole -= 1;
                self.round_score -= result;
                self.total_score -= result;
                return_vec.append(&mut self.shift_scores(true));
            } else {
                return_vec.push(self.set_tot_score());
                if self.round_ind > 0 {
                    return_vec.append(&mut self.set_round_score());
                }
            }
        }
        return_vec
    }

    fn set_tot_score(&self) -> JsString {
        VMixFunction::SetText{
            value: fix_score(self.total_score),
            input: VmixProperty::TotalScore(self.ind).into(),
        }
            .to_cmd()
            .into()
    }

    fn hide_round_score(&self) -> JsString {
        VMixFunction::SetTextVisibleOff{
            input: VmixProperty::RoundScore(self.ind).into(),
        }
            .to_cmd()
            .into()
    }

    fn set_round_score(&self) -> Vec<JsString> {
        vec![
            VMixFunction::SetText{
                value: "(".to_string() + &fix_score(self.round_score) + ")",
                input: VmixProperty::RoundScore(self.ind).into(),
            }
                .to_cmd()
                .into(),
            self.show_round_score(),
        ]
    }

    fn show_round_score(&self) -> JsString {
        VMixFunction::SetTextVisibleOn{
            input: VmixProperty::RoundScore(self.ind).into(),
        }
            .to_cmd()
            .into()
    }

    pub fn set_pos(&self) -> JsString {
        let value_string = if self.lb_even {
            "T".to_string()
        } else {
            "".to_string()
        } + &self.lb_pos.to_string();

        if self.visible_player {
            VMixFunction::SetText{
                value: value_string,
                input: VmixProperty::PlayerPosition(self.ind as u16).into(),
            }
                .to_cmd()
                .into()
        } else {
            "".into()
        }
    }

    pub fn hide_pos(&mut self) -> Vec<JsString> {
        self.pos_visible = false;
        vec![
            VMixFunction::SetTextVisibleOff{
                input: VmixProperty::PlayerPosition(self.ind as u16).into(),
            }
                .to_cmd()
                .into(),
            VMixFunction::SetColor{
                color: "00000000".to_string(),
                input: VmixProperty::PosRightTriColor(self.ind).into(),
            }
                .to_cmd()
                .into(),
            VMixFunction::SetColor{
                color: "00000000".to_string(),
                input: VmixProperty::PosSquareColor(self.ind).into(),
            }
                .to_cmd()
                .into(),
        ]
    }

    pub fn show_pos(&mut self) -> Vec<JsString> {
        self.pos_visible = true;
        vec![
            VMixFunction::SetTextVisibleOn{
                input: VmixProperty::PlayerPosition(self.ind as u16).into(),
            }
                .to_cmd()
                .into(),
            VMixFunction::SetColor{
                color: DEFAULT_BACKGROUND_COL.to_string(),
                input: VmixProperty::PosRightTriColor(self.ind).into(),
            }
                .to_cmd()
                .into(),
            VMixFunction::SetColor{
                color: DEFAULT_BACKGROUND_COL.to_string(),
                input: VmixProperty::PosSquareColor(self.ind).into(),
            }
                .to_cmd()
                .into(),
        ]
    }

    pub fn toggle_pos(&mut self) -> Vec<JsString> {
        if self.pos_visible {
            self.hide_pos()
        } else {
            self.show_pos()
        }
    }

    pub fn shift_scores(&mut self, last_blank: bool) -> Vec<JsString> {
        let mut return_vec = vec![];
        let in_hole = self.hole;

        let diff = self.hole - 8 + {
            if last_blank && self.hole != 17 {
                1
            } else {
                0
            }
        };

        self.hole = diff;
        self.shift = diff;
        for _ in diff..=in_hole {
            return_vec.append(&mut self.set_hole_score());
        }
        if last_blank && self.hole != 18 {
            return_vec.push(
                VMixFunction::SetText{
                    value: (in_hole + 2).to_string(),
                    input: VmixProperty::HoleNumber(9, self.ind).into(),
                }
                    .to_cmd()
                    .into(),
            );
            return_vec.push(
                VMixFunction::SetTextVisibleOff{
                    input: VmixProperty::Score(9, self.ind).into(),
                }
                    .to_cmd()
                    .into(),
            );
            return_vec.push(
                VMixFunction::SetColor{
                    color: DEFAULT_FOREGROUND_COL.to_string() + "00",
                    input: VmixProperty::ScoreColor(9, self.ind).into(),
                }
                    .to_cmd()
                    .into(),
            );
        }
        if self.visible_player {
            return_vec
        } else {
            vec![]
        }
    }

    fn del_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        let score_prop = VmixProperty::Score(self.hole + 1, self.ind);
        let col_prop = VmixProperty::ScoreColor(self.hole + 1, self.ind);
        let h_num_prop = VmixProperty::HoleNumber(self.hole + 1, self.ind);
        return_vec.push(
            VMixFunction::SetText{
                value: "".to_string(),
                input: score_prop.clone().into(),
            }
                .to_cmd()
                .into(),
        );

        return_vec.push(
            VMixFunction::SetColor{
                color: DEFAULT_FOREGROUND_COL.to_string() + "00",
                input: col_prop.into(),
            }
                .to_cmd()
                .into(),
        );

        return_vec.push(
            VMixFunction::SetText{
                value: (self.hole + 1).to_string(),
                input: h_num_prop.into(),
            }
                .to_cmd()
                .into(),
        );
        return_vec.push(
            VMixFunction::SetTextVisibleOff{
                input: score_prop.into(),
            }
                .to_cmd()
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
        self.shift = 0;
        self.round_score = 0;
        self.total_score = self.score_before_round();

        return_vec.push(self.set_tot_score());
        return_vec.append(&mut self.hide_pos());
        if self.round_ind > 0 {
            return_vec.append(&mut self.set_round_score());
        } else {
            self.total_score = 0;
            return_vec.push(self.hide_round_score());
        }
        return_vec
    }

    pub fn set_throw(&self) -> JsString {
        VMixFunction::SetText{
            value: self.throws.to_string(),
            input: VmixProperty::Throw(self.ind).into(),
        }
            .to_cmd()
            .into()
    }

    pub fn start_score_anim(&mut self) -> Vec<JsString> {
        let return_vec: Vec<JsString> = vec![
            VMixFunction::<VmixProperty>::OverlayInput4Off.to_cmd().into(),
            self.set_input_pan(),
            VMixFunction::<VmixProperty>::OverlayInput4(self.get_mov()).to_cmd().into(),
        ];
        self.ob = false;
        return_vec
    }

    fn set_input_pan(&mut self) -> JsString {
        let pan = match self.ind + 1 {
            1 => -0.628,
            2 => -0.628 + 0.419,
            3 => -0.628 + 0.4185 * 2.0,
            4 => -0.628 + 0.419 * 3.0,
            _ => -0.628,
        };
        VMixFunction::SetPanX{
            value: pan,
            input: VmixProperty::Mov(self.get_mov()).into(),
        }
            .to_cmd()
            .into()
    }

    fn get_mov(&self) -> String {
        if self.ob {
            "50 ob.mov".to_string()
        } else {
            match self.current_round() {
                Some(round) => match round.results.get(self.hole) {
                    Some(result) => result.get_mov().to_string(),
                    None => "".to_string(),
                },
                None => "".to_string(),
            }
        }
    }

    pub fn set_round(&mut self, round_ind: usize) -> Vec<JsString> {
        self.round_ind = round_ind;
        self.reset_scores()
    }

    // LB TCP
    fn set_lb_pos(&mut self) -> JsString {
        let thing: JsString = VMixFunction::SetText{
            value: self.position.to_string(),
            input: LeaderBoardProperty::Position{pos:self.position, lb_pos: self.lb_pos, tied: self.lb_even}.into(),
        }
            .to_cmd()
            .into();
        thing
    }

    fn set_lb_name(&self) -> JsString {
        VMixFunction::SetText{
            value: self.name.clone(),
            input: LeaderBoardProperty::Name(self.position).into(),
        }
            .to_cmd()
            .into()
    }

    fn set_lb_hr(&self) -> JsString {
        let value = if self.hot_round && self.round_ind != 0 && self.hole != 0 && self.hole < 19 {
            r"X:\FLIPUP\grafik\fire.png"
        } else {
            r"X:\FLIPUP\grafik\alpha.png"
        };
        VMixFunction::SetImage{
            value: value.to_string(),
            input: LeaderBoardProperty::HotRound(self.position).into(),
        }
            .to_cmd()
            .into()
    }

    fn set_rs(&self, hidden: bool) -> JsString {
        VMixFunction::SetText{
            value: if self.lb_pos == 0 {
                "".to_string()
            } else if hidden {
                "E".to_string()
            } else {
                fix_score(self.round_score)
            },
            input: LeaderBoardProperty::RoundScore(self.position).into(),
        }
            .to_cmd()
            .into()
    }

    fn set_ts(&self, hidden: bool) -> Vec<JsString> {
        let mut r_vec: Vec<JsString> = vec![
            VMixFunction::SetTextVisibleOn{
                input: LeaderBoardProperty::TotalScore{pos:self.position}.into(),
            }
                .to_cmd()
                .into(),
            VMixFunction::SetTextVisibleOn{
                input: LeaderBoardProperty::TotalScoreTitle.into(),
            }
                .to_cmd()
                .into(),
        ];
        r_vec.push(
            VMixFunction::SetText{
                value: if self.lb_pos == 0 {
                    "".to_string()
                } else if hidden && self.round_ind == 0{
                    "E".to_string()
                } else {
                    fix_score(self.total_score)
                },
                input: LeaderBoardProperty::TotalScore{pos:self.position}.into(),
            }
                .to_cmd()
                .into(),
        );
        r_vec
    }

    fn set_moves(&self) -> Vec<JsString> {
        self.rank.get_tcps(self.position)
    }

    fn set_thru(&self, hidden: bool) -> JsString {
        VMixFunction::SetText{
            value: if self.lb_pos == 0 {
                "".to_string()
            } else if hidden {
                "0".to_string()
            } else {
                self.thru.to_string()
            },
            input: LeaderBoardProperty::Thru(self.position).into(),
        }
            .to_cmd()
            .into()
    }

    pub fn set_lb(&mut self) -> Vec<JsString> {
        self.check_if_allowed_to_visible();
        let hide = self.thru == 0;
        let mut return_vec: Vec<JsString> = vec![
            self.set_lb_pos(),
            self.set_lb_name(),
            self.set_lb_hr(),
            self.set_rs(hide),
        ];

        return_vec.append(&mut self.set_ts(hide));
        return_vec.append(&mut self.set_moves());

        return_vec.push(self.set_thru(hide));
        return_vec
    }
}

#[derive(Clone)]
pub struct RustHandler {
    pub chosen_division: cynic::Id,
    event: queries::Event,
    divisions: Vec<queries::Division>,
    round_ind: usize,
    vmix_id: String,
    lb_vmix_id: String,
}

impl RustHandler {
    pub fn new(
        pre_event: GraphQlResponse<queries::EventQuery>,
        vmix_id: String,
        lb_vmix_id: String,
    ) -> Self {
        let event = pre_event.data.expect("no data").event.expect("no event");
        let mut divisions: Vec<queries::Division> = vec![];
        event
            .divisions
            .iter()
            .flatten()
            .for_each(|div| divisions.push(div.clone()));

        Self {
            chosen_division: divisions.first().expect("NO DIV CHOSEN").id.clone(),
            event,
            divisions,
            round_ind: 0,
            vmix_id,
            lb_vmix_id,
        }
    }

    pub fn get_divisions(&self) -> Vec<Division> {
        let mut divs: Vec<Division> = vec![];
        for div in &self.divisions {
            divs.push(div.clone());
        }
        divs
    }

    pub fn get_players(&self) -> Vec<NewPlayer> {
        let mut players: Vec<queries::PoolLeaderboardPlayer> = vec![];
        let mut out_vec: Vec<NewPlayer> = vec![];

        let len_of_pools = self.event.rounds[self.round_ind]
            .clone()
            .expect("no round")
            .pools
            .len();
        for ind in 0..len_of_pools {
            for div in &self.event.rounds[self.round_ind]
                .clone()
                .expect("no round")
                .pools[ind]
                .clone()
                .leaderboard
                .expect("no leaderboard")
            {
                match div {
                    Some(queries::PoolLeaderboardDivisionCombined::Pld(division)) => {
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
                self.vmix_id.clone(),
                self.lb_vmix_id.clone(),
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
    }
}
#[allow(non_snake_case, non_camel_case_types)]
mod schema {
    cynic::use_schema!(r#"src/schema.graphql"#);
}
