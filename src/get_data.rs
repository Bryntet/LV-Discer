use cynic::GraphQlResponse;

use self::queries::Division;
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

const DEFAULT_FOREGROUND_COL: &str = "3F334D";
const DEFAULT_BACKGROUND_COL: &str = "574B60";
#[derive(Debug, Clone, Default)]
pub enum RankUpDown {
    Up(i16),
    Down(i16),
    #[default]
    Same,
}

impl RankUpDown {
    fn get_tcps(&self, pos: u16, id: &str) -> Vec<JsString> {
        let the_vec = vec![
            self.make_move(pos, id).into(),
            self.make_arrow(pos, id).into(),
        ];
        //log(&format!("{:#?}", the_vec));
        the_vec
    }

    fn make_move(&self, pos: u16, id: &str) -> String {
        let movement = match self {
            RankUpDown::Up(val) => val.to_string(),
            RankUpDown::Down(val) => val.to_string(),
            RankUpDown::Same => "".to_string(),
        };

        VmixFunction::SetText(VmixInfo {
            id,
            value: movement,
            prop: VmixProperty::LBMove(pos),
        })
        .to_cmd()
    }

    fn make_arrow(&self, pos: u16, id: &str) -> String {
        let img = match self {
            RankUpDown::Up(_) => r"x:\FLIPUP\grafik\greentri.png",
            RankUpDown::Down(_) => r"x:\FLIPUP\grafik\redtri.png",
            RankUpDown::Same => r"x:\FLIPUP\grafik\alpha.png",
        };

        VmixFunction::SetImage(VmixInfo {
            id,
            value: img.to_string(),
            prop: VmixProperty::LBArrow(pos),
        })
        .to_cmd()
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
        if hole == 0 {
            0
        } else {
            (0..hole + 1).map(|i| self.hole_score(i)).sum()
        }
    }

    fn hole_score(&self, hole: usize) -> i16 {
        self.results[hole].actual_score() as i16
    }

    pub fn get_hole_info(&self, hole: usize) -> Vec<JsString> {
        let id = "0e76d38f-6e8d-4f7d-b1a6-e76f695f2094";

        let mut r_vec: Vec<JsString> = vec![];
        let hole = &self.results[hole].hole;
        r_vec.push(
            VmixFunction::SetText(VmixInfo {
                id,
                value: hole.number.to_string(),
                prop: VmixProperty::Hole,
            })
            .to_cmd()
            .into(),
        );

        r_vec.push(
            VmixFunction::SetText(VmixInfo {
                id,
                value: hole.par.unwrap().to_string(),
                prop: VmixProperty::HolePar,
            })
            .to_cmd()
            .into(),
        );
        let meters = if hole.measure_in_meters.unwrap_or(false) {
            hole.length.unwrap_or(0.0)
        } else {
            hole.length.unwrap_or(0.0) * 0.9144
        };

        r_vec.push(
            VmixFunction::SetText(VmixInfo {
                id,
                value: (meters as u64).to_string() + "M",
                prop: VmixProperty::HoleMeters,
            })
            .to_cmd()
            .into(),
        );

        let feet = (meters * 3.28084) as u64;
        r_vec.push(
            VmixFunction::SetText(VmixInfo {
                id,
                value: feet.to_string() + "FT",
                prop: VmixProperty::HoleFeet,
            })
            .to_cmd()
            .into(),
        );
        r_vec
    }
}

pub struct VmixInfo<'a> {
    pub id: &'a str,
    pub value: String,
    pub prop: VmixProperty,
}

pub enum VmixFunction<'a> {
    SetText(VmixInfo<'a>),
    SetPanX(VmixInfo<'a>),
    SetColor(VmixInfo<'a>),
    SetTextVisibleOn(VmixInfo<'a>),
    SetTextVisibleOff(VmixInfo<'a>),
    SetImage(VmixInfo<'a>),
    Restart(String),
    Play(String),
    OverlayInput4Off,
    OverlayInput4(String),
}

impl VmixFunction<'_> {
    pub fn to_cmd(&self) -> String {
        match self {
            VmixFunction::SetText(info) => {
                match info.prop {
                    VmixProperty::Score(_, _) => {
                        return format!(
                            "FUNCTION SetText Value={}&Input={}&{}",
                            fix_score(info.value.parse::<i16>().unwrap_or(0)),
                            info.id,
                            info.prop.selection()
                        );
                    }
                    VmixProperty::LBPosition(_, _, _) => {
                        return format!(
                            "FUNCTION SetText Input={}&{}",
                            info.id,
                            info.prop.selection()
                        );
                    }
                    _ => {}
                }
                format!(
                    "FUNCTION SetText Value={}&Input={}&{}",
                    info.value,
                    info.id,
                    info.prop.selection()
                )
            }
            VmixFunction::SetPanX(info) => {
                format!("FUNCTION SetPanX Value={}&Input={}", info.value, info.id,)
            }
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
                "FUNCTION SetColor Value=#{}&Input={}&{}",
                info.value,
                info.id,
                info.prop.selection()
            ),
            VmixFunction::SetImage(info) => format!(
                "FUNCTION SetImage Value={}&Input={}&{}",
                info.value,
                info.id,
                info.prop.selection()
            ),
            VmixFunction::OverlayInput4Off => "FUNCTION OverlayInput4Off".to_string(),
            VmixFunction::OverlayInput4(mov) => format!("FUNCTION OverlayInput4 Input={}", mov),
        }
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

#[derive(Clone)]
pub enum VmixProperty {
    Score(usize, usize),
    HoleNumber(usize, usize),
    ScoreColor(usize, usize),
    PosRightTriColor(usize),
    PosSquareColor(usize),
    Name(usize),
    TotalScore(usize),
    RoundScore(usize),
    Throw(usize),
    Mov(String),
    PlayerPosition(u16),
    LBPosition(u16, u16, bool),
    LBName(u16),
    LBHotRound(u16),
    Lbrs(u16),
    Lbts(u16),
    LbtsTitle,
    LBMove(u16),
    LBArrow(u16),
    LBThru(u16),
    LBCheckinText,
    HoleMeters,
    HoleFeet,
    HolePar,
    Hole,
}

impl VmixProperty {
    fn selection(&self) -> String {
        match self {
            VmixProperty::Score(v1, v2) => format!("SelectedName=s{}p{}.Text", v1, v2 + 1),
            VmixProperty::HoleNumber(v1, v2) => {
                format!("SelectedName=HN{}p{}.Text", v1, v2 + 1)
            }
            VmixProperty::ScoreColor(v1, v2) => {
                format!("SelectedName=h{}p{}.Fill.Color", v1, v2 + 1)
            }
            VmixProperty::PosRightTriColor(v1) => {
                format!("SelectedName=rghtri{}.Fill.Color", v1 + 1)
            }
            VmixProperty::PosSquareColor(v1) => format!("SelectedName=rekt{}.Fill.Color", v1 + 1),
            VmixProperty::Name(ind) => format!("SelectedName=namep{}.Text", ind + 1),
            VmixProperty::TotalScore(ind) => format!("SelectedName=scoretotp{}.Text", ind + 1),
            VmixProperty::RoundScore(ind) => format!("SelectedName=scorerndp{}.Text", ind + 1),
            VmixProperty::Throw(ind) => format!("SelectedName=t#p{}.Text", ind + 1),
            VmixProperty::Mov(id) => format!("SelectedName={}", id),
            VmixProperty::PlayerPosition(pos) => format!("SelectedName=posp{}.Text", pos + 1),
            VmixProperty::LBPosition(pos, lb_pos, tied) => {
                if *tied && lb_pos != &0 {
                    format!("SelectedName=pos#{}.Text&Value=T{}", pos, lb_pos)
                } else {
                    format!("SelectedName=pos#{}.Text&Value={}", pos, if lb_pos != &0 {lb_pos.to_string()} else {"".to_string()})
                }
            }
            VmixProperty::LBName(pos) => format!("SelectedName=name#{}.Text", pos),
            VmixProperty::LBHotRound(pos) => format!("SelectedName=hrp{}.Source", pos),
            VmixProperty::Lbrs(pos) => format!("SelectedName=rs#{}.Text", pos),
            VmixProperty::Lbts(pos) => format!("SelectedName=ts#{}.Text", pos),
            VmixProperty::LbtsTitle => "SelectedName=ts.Text".to_string(),
            VmixProperty::LBMove(pos) => format!("SelectedName=move{}.Text", pos),
            VmixProperty::LBArrow(pos) => format!("SelectedName=arw{}.Source", pos),
            VmixProperty::LBThru(pos) => format!("SelectedName=thru#{}.Text", pos),
            VmixProperty::LBCheckinText => "SelectedName=checkintext.Text".to_string(),
            VmixProperty::HoleMeters => "SelectedName=meternr.Text".to_string(),
            VmixProperty::HoleFeet => "SelectedName=feetnr.Text".to_string(),
            VmixProperty::HolePar => "SelectedName=parnr.Text".to_string(),
            VmixProperty::Hole => "SelectedName=hole.Text".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewPlayer {
    pub player_id: cynic::Id,
    pub name: String,
    pub rank: RankUpDown,
    best_score: bool,
    pub total_score: i16, // Total score for all rounds
    pub round_score: i16, // Score for only current round
    round_ind: usize,
    pub rounds: Vec<PlayerRound>,
    div_id: cynic::Id,
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
}

impl Default for NewPlayer {
    fn default() -> Self {
        Self {
            player_id: cynic::Id::from(""),
            name: "".to_string(),
            rank: RankUpDown::Same,
            best_score: false,
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
            best_score: false,
            rounds,
            div_id,
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
        self.current_round().results[self.hole]
            .get_score_colour()
            .into()
    }

    pub fn current_round(&self) -> &PlayerRound {
        if self.round_ind >= self.rounds.len() {
            &self.rounds[self.rounds.len() - 1]
        } else {
            &self.rounds[self.round_ind]
        }
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

    pub fn set_name(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.vmix_id,
            value: self.name.clone(),
            prop: VmixProperty::Name(self.ind),
        })
        .to_cmd()
        .into()
    }

    pub fn set_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        // log(&format!("{}", self.round_ind));
        // log(&format!("{:#?}", self.rounds));
        let result = self.current_round().hole_score(self.hole);

        self.make_tot_score();
        // Set score
        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: result.to_string(),
                prop: VmixProperty::Score(self.hole + 1 - self.shift, self.ind),
            })
            .to_cmd()
            .into(),
        );
        // Set colour
        return_vec.push(
            VmixFunction::SetColor(VmixInfo {
                id: &self.vmix_id,
                value: self.get_col(),
                prop: VmixProperty::ScoreColor(self.hole + 1 - self.shift, self.ind),
            })
            .to_cmd()
            .into(),
        );
        // Show score
        return_vec.push(
            VmixFunction::SetTextVisibleOn(VmixInfo {
                id: &self.vmix_id,
                value: self.get_col(),
                prop: VmixProperty::Score(self.hole + 1 - self.shift, self.ind),
            })
            .to_cmd()
            .into(),
        );

        // HoleNumber
        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: (self.hole + 1).to_string(),
                prop: VmixProperty::HoleNumber(self.hole + 1 - self.shift, self.ind),
            })
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

        return_vec
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
        VmixFunction::SetText(VmixInfo {
            id: &self.vmix_id,
            value: fix_score(self.total_score),
            prop: VmixProperty::TotalScore(self.ind),
        })
        .to_cmd()
        .into()
    }

    fn hide_round_score(&self) -> JsString {
        VmixFunction::SetTextVisibleOff(VmixInfo {
            id: &self.vmix_id,
            value: "".to_string(),
            prop: VmixProperty::RoundScore(self.ind),
        })
        .to_cmd()
        .into()
    }

    fn set_round_score(&self) -> Vec<JsString> {
        vec![
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: "(".to_string() + &fix_score(self.round_score) + ")",
                prop: VmixProperty::RoundScore(self.ind),
            })
            .to_cmd()
            .into(),
            self.show_round_score(),
        ]
    }

    fn show_round_score(&self) -> JsString {
        VmixFunction::SetTextVisibleOn(VmixInfo {
            id: &self.vmix_id,
            value: "".to_string(),
            prop: VmixProperty::RoundScore(self.ind),
        })
        .to_cmd()
        .into()
    }

    pub fn set_pos(&self) -> JsString {
        let value_string = if self.lb_even {
            "T".to_string()
        } else {
            "".to_string()
        } + &self.lb_pos.to_string();

        VmixFunction::SetText(VmixInfo {
            id: &self.vmix_id,
            value: value_string,
            prop: VmixProperty::PlayerPosition(self.ind as u16 ),
        })
        .to_cmd()
        .into()
    }

    pub fn hide_pos(&mut self) -> Vec<JsString> {
        self.pos_visible = false;
        vec![
            VmixFunction::SetTextVisibleOff(VmixInfo {
                id: &self.vmix_id,
                value: "".to_string(),
                prop: VmixProperty::PlayerPosition(self.ind as u16),
            })
            .to_cmd()
            .into(),
            VmixFunction::SetColor(VmixInfo {
                id: &self.vmix_id,
                value: "00000000".to_string(),
                prop: VmixProperty::PosRightTriColor(self.ind ),
            })
            .to_cmd()
            .into(),
            VmixFunction::SetColor(VmixInfo {
                id: &self.vmix_id,
                value: "00000000".to_string(),
                prop: VmixProperty::PosSquareColor(self.ind),
            })
            .to_cmd()
            .into(),
        ]
    }

    pub fn show_pos(&mut self) -> Vec<JsString> {
        self.pos_visible = true;
        vec![
            VmixFunction::SetTextVisibleOn(VmixInfo {
                id: &self.vmix_id,
                value: "".to_string(),
                prop: VmixProperty::PlayerPosition(self.ind as u16),
            })
            .to_cmd()
            .into(),
            VmixFunction::SetColor(VmixInfo {
                id: &self.vmix_id,
                value: DEFAULT_BACKGROUND_COL.to_string(),
                prop: VmixProperty::PosRightTriColor(self.ind),
            })
            .to_cmd()
            .into(),
            VmixFunction::SetColor(VmixInfo {
                id: &self.vmix_id,
                value: DEFAULT_BACKGROUND_COL.to_string(),
                prop: VmixProperty::PosSquareColor(self.ind),
            })
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
        let in_hole = self.hole.clone();

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
                VmixFunction::SetText(VmixInfo {
                    id: &self.vmix_id,
                    value: (in_hole + 2).to_string(),
                    prop: VmixProperty::HoleNumber(9, self.ind),
                })
                .to_cmd()
                .into(),
            );
            return_vec.push(
                VmixFunction::SetTextVisibleOff(VmixInfo {
                    id: &self.vmix_id,
                    value: "".to_string(),
                    prop: VmixProperty::Score(9, self.ind),
                })
                .to_cmd()
                .into(),
            );
            return_vec.push(
                VmixFunction::SetColor(VmixInfo {
                    id: &self.vmix_id,
                    value: DEFAULT_FOREGROUND_COL.to_string(),
                    prop: VmixProperty::ScoreColor(9, self.ind),
                })
                .to_cmd()
                .into(),
            );
        }
        return_vec
    }

    fn del_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        let score_prop = VmixProperty::Score(self.hole + 1, self.ind);
        let col_prop = VmixProperty::ScoreColor(self.hole + 1, self.ind);
        let h_num_prop = VmixProperty::HoleNumber(self.hole + 1, self.ind);
        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: "".to_string(),
                prop: score_prop.clone(),
            })
            .to_cmd()
            .into(),
        );

        return_vec.push(
            VmixFunction::SetColor(VmixInfo {
                id: &self.vmix_id,
                value: DEFAULT_FOREGROUND_COL.to_string(),
                prop: col_prop,
            })
            .to_cmd()
            .into(),
        );

        return_vec.push(
            VmixFunction::SetText(VmixInfo {
                id: &self.vmix_id,
                value: (self.hole + 1).to_string(),
                prop: h_num_prop,
            })
            .to_cmd()
            .into(),
        );
        return_vec.push(
            VmixFunction::SetTextVisibleOff(VmixInfo {
                id: &self.vmix_id,
                value: "".to_string(),
                prop: score_prop,
            })
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
        VmixFunction::SetText(VmixInfo {
            id: &self.vmix_id,
            value: self.throws.to_string(),
            prop: VmixProperty::Throw(self.ind),
        })
        .to_cmd()
        .into()
    }

    pub fn start_score_anim(&mut self) -> Vec<JsString> {
        let return_vec: Vec<JsString> = vec![
            VmixFunction::OverlayInput4Off.to_cmd().into(),
            self.set_input_pan(),
            VmixFunction::OverlayInput4(self.get_mov()).to_cmd().into(),
        ];
        self.ob = false;
        // TODO!: REMOVE THIS AAAAAH
        if self.ind + 1 != 4 {
            return_vec
        } else {
            vec![]
        }
    }

    fn set_input_pan(&mut self) -> JsString {
        let pan = match self.ind + 1 {
            1 => -0.628,
            2 => -0.628 + 0.419,
            3 => -0.628 + 0.4185 * 2.0,
            4 => -0.628 + 0.419 * 3.0,
            _ => -0.628,
        };
        VmixFunction::SetPanX(VmixInfo {
            id: &self.get_mov(),
            value: pan.to_string(),
            prop: VmixProperty::Mov(self.get_mov()),
        })
        .to_cmd()
        .into()
    }

    fn play_anim(&mut self) -> Vec<JsString> {
        vec![
            VmixFunction::Restart(self.get_mov()).to_cmd().into(),
            //VmixFunction::Play(self.get_mov()).to_string().into(),
        ]
    }

    fn get_mov(&self) -> String {
        if self.ob {
            "50 ob.mov".to_string()
        } else {
            self.current_round().results[self.hole]
                .get_mov()
                .to_string()
        }
    }

    pub fn set_round(&mut self, round_ind: usize) -> Vec<JsString> {
        self.round_ind = round_ind;
        self.reset_scores()
    }

    // LB TCP
    fn set_lb_pos(&mut self) -> JsString {
        let thing: JsString = VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: self.position.to_string(),
            prop: VmixProperty::LBPosition(self.position, self.lb_pos, self.lb_even),
        })
        .to_cmd()
        .into();
        thing
    }

    fn set_lb_name(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: self.name.clone(),
            prop: VmixProperty::LBName(self.position),
        })
        .to_cmd()
        .into()
    }

    fn set_lb_hr(&self) -> JsString {
        
        let value = if self.hot_round && self.round_ind != 0 && self.hole != 0 && self.hole < 19 {
            r"X:\FLIPUP\grafik\fire.png"
        } else {
            r"X:\FLIPUP\grafik\alpha.png"
        };
        VmixFunction::SetImage(VmixInfo {
            id: &self.lb_vmix_id,
            value: value.to_string(),
            prop: VmixProperty::LBHotRound(self.position),
        })
        .to_cmd()
        .into()
    }

    fn set_rs(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: if self.lb_pos != 0 {fix_score(self.round_score)} else {"".to_string()},
            prop: VmixProperty::Lbrs(self.position),
        })
        .to_cmd()
        .into()
    }

    fn set_ts(&self) -> Vec<JsString> {

        let mut r_vec: Vec<JsString> = if self.round_ind == 0 {
            vec![VmixFunction::SetTextVisibleOff(VmixInfo {
                id: &self.lb_vmix_id,
                value: "".to_string(),
                prop: VmixProperty::Lbts(self.position),
            })
            .to_cmd()
            .into(),
            VmixFunction::SetTextVisibleOff(VmixInfo { id: &self.lb_vmix_id, value: "".to_string(), prop: VmixProperty::LbtsTitle }).to_cmd().into()]
        } else {
            vec![VmixFunction::SetTextVisibleOn(VmixInfo {
                id: &self.lb_vmix_id,
                value: "".to_string(),
                prop: VmixProperty::Lbts(self.position),
            }).to_cmd().into(),
            VmixFunction::SetTextVisibleOn(VmixInfo { id: &self.lb_vmix_id, value: "".to_string(), prop: VmixProperty::LbtsTitle }).to_cmd().into()
            ]
        };

        r_vec.push(VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: fix_score(self.total_score),
            prop: VmixProperty::Lbts(self.position),
        })
        .to_cmd()
        .into());
        r_vec
    }

    fn set_moves(&self) -> Vec<JsString> {
        self.rank.get_tcps(self.position, &self.lb_vmix_id)
    }

    fn set_thru(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: if self.hole == 0 {"".to_string()} else {(self.hole + 1).to_string()},
            prop: VmixProperty::LBThru(self.position),
        })
        .to_cmd()
        .into()
    }

    pub fn set_lb(&mut self) -> Vec<JsString> {
        self.check_if_allowed_to_visible();
        let mut return_vec: Vec<JsString> = vec![
            self.set_lb_pos(),
            self.set_lb_name(),
            self.set_lb_hr(),
            self.set_rs(),
        ];

        return_vec.append(&mut self.set_ts());
        return_vec.append(&mut self.set_moves());
        return_vec.push(self.set_thru());
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

    pub fn get_round(&self) -> queries::Round {
        self.event.rounds[self.round_ind].clone().expect("no round")
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

    fn assign_position(players: &mut Vec<NewPlayer>) {
        // Sort players in descending order by total_score
        players.sort_unstable_by(|a, b| b.total_score.cmp(&a.total_score));

        // Iterate over sorted players to assign position
        let mut current_position: u16 = 1;
        let mut last_score: i16 = players[0].total_score;
        for player in players.iter_mut() {
            // If current player's score is less than last score, increment position
            if player.total_score < last_score {
                current_position += 1;
            }

            // Assign current position to player
            player.position = current_position;

            // Update last score
            last_score = player.total_score;
        }
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

    #[derive(Default)]
    struct LBInformation {
        position: u8,
        round_score: i16,
        total_score: i16,
        player_name: String,

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
        pub measure_in_meters: Option<bool>,
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
