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
pub enum RankUpDown {
    Up(i16),
    Down(i16),
    Same,
}
impl Default for RankUpDown {
    fn default() -> Self {
        RankUpDown::Same
    }
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
            id: id,
            value: movement,
            prop: VmixProperty::LBMove(pos),
        })
        .to_string()
    }

    fn make_arrow(&self, pos: u16, id: &str) -> String {
        let img = match self {
            RankUpDown::Up(_) => r"x:\FLIPUP\grafik\greentri.png",
            RankUpDown::Down(_) => r"x:\FLIPUP\grafik\redtri.png",
            RankUpDown::Same => r"x:\FLIPUP\grafik\alpha.png",
        };

        VmixFunction::SetImage(VmixInfo {
            id: id,
            value: img.to_string(),
            prop: VmixProperty::LBArrow(pos),
        })
        .to_string()
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
        (0..hole + 1).map(|i| self.hole_score(i)).sum()
    }

    fn hole_score(&self, hole: usize) -> i16 {
        return self.results[hole].actual_score() as i16;
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
    pub fn to_string(&self) -> String {
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
                    VmixProperty::TotalScore(_, rs, ts) => {
                        return format!(
                            "FUNCTION SetText Value=({}) {}()&Input={}&{}",
                            fix_score(rs),
                            fix_score(ts),
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
                return format!(
                    "FUNCTION SetText Value={}&Input={}&{}",
                    info.value,
                    info.id,
                    info.prop.selection()
                );
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
    if score == 0 {
        return "E".to_string();
    } else if score > 0 {
        return format!("+{}", score);
    } else {
        return format!("{}", score);
    }
}

#[derive(Clone)]
pub enum VmixProperty {
    Score(usize, usize),
    HoleNumber(usize, usize),
    Color(usize, usize),
    Name(usize),
    TotalScore(usize, i16, i16),
    Throw(usize),
    Mov(String),
    LBPosition(u16, u16, bool),
    LBName(u16),
    LBHotRound(u16),
    LBRS(u16),
    LBTS(u16),
    LBMove(u16),
    LBArrow(u16),
    LBThru(u16),
    LBCheckinText(),
}

impl VmixProperty {
    fn selection(&self) -> String {
        match self {
            VmixProperty::Score(v1, v2) => format!("SelectedName=s{}p{}.Text", v1, v2 + 1),
            VmixProperty::HoleNumber(v1, v2) => {
                format!("SelectedName=HN{}p{}.Text", v1, v2 + 1)
            }
            VmixProperty::Color(v1, v2) => format!("SelectedName=h{}p{}.Fill.Color", v1, v2 + 1),
            VmixProperty::Name(ind) => format!("SelectedName=namep{}.Text", ind + 1),
            VmixProperty::TotalScore(ind, ..) => format!("SelectedName=scoretotp{}.Text", ind + 1),
            VmixProperty::Throw(ind) => format!("SelectedName=t#p{}.Text", ind + 1),
            VmixProperty::Mov(id) => format!("SelectedName={}", id),
            VmixProperty::LBPosition(pos, lb_pos, tied) => {
                if *tied {
                    format!("SelectedName=pos#{}.Text&Value=T{}", pos, lb_pos)
                } else {
                    format!("SelectedName=pos#{}.Text&Value={}", pos, lb_pos)
                }
            }
            VmixProperty::LBName(pos) => format!("SelectedName=name#{}.Text", pos),
            VmixProperty::LBHotRound(pos) => format!("SelectedName=hrp{}.Source", pos),
            VmixProperty::LBRS(pos) => format!("SelectedName=rs#{}.Text", pos),
            VmixProperty::LBTS(pos) => format!("SelectedName=ts#{}.Text", pos),
            VmixProperty::LBMove(pos) => format!("SelectedName=move{}.Text", pos),
            VmixProperty::LBArrow(pos) => format!("SelectedName=arw{}.Source", pos),
            VmixProperty::LBThru(pos) => format!("SelectedName=thru#{}.Text", pos),
            VmixProperty::LBCheckinText() => format!("SelectedName=checkintext.Text"),
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
    lb_vmix_id: String,
    pub lb_pos: u16,
    pub old_pos: u16,
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
            vmix_id,
            lb_vmix_id,
            ..Default::default()
        }
    }

    pub fn get_round_total_score(&self, round_ind: usize) -> i16 {
        self.current_round().score_to_hole(17)
    }

    pub fn score_before_round(&self) -> i16 {
        let mut total_score = 0;
        for round_ind in 0..self.round_ind {
            total_score += self.rounds[round_ind].score_to_hole(17)
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
        &self.rounds[self.round_ind]
    }

    pub fn make_tot_score(&mut self) {
        self.round_score = self.current_round().score_to_hole(self.hole);
        self.total_score = self.score_before_round() + self.round_score;
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
        .to_string()
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
            .to_string()
            .into(),
        );
        log(&format!("{} {}", self.hole, self.shift));
        // Set colour
        return_vec.push(
            VmixFunction::SetColor(VmixInfo {
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
                value: (self.hole + 1).to_string(),
                prop: VmixProperty::HoleNumber(self.hole + 1 - self.shift, self.ind),
            })
            .to_string()
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
            prop: VmixProperty::TotalScore(self.ind, self.round_score, self.total_score),
        })
        .to_string()
        .into()
    }

    pub fn shift_scores(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        let in_hole = self.hole.clone();
        let diff = self.hole - 8;
        self.hole = diff;
        self.shift = diff;
        for _ in diff..in_hole {
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
                prop: score_prop.clone(),
            })
            .to_string()
            .into(),
        );

        return_vec.push(
            VmixFunction::SetColor(VmixInfo {
                id: &self.vmix_id,
                value: DEFAULT_BG_COL.to_string(),
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
        self.total_score = self.score_before_round();
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
        //return_vec.append(&mut self.play_anim());
        return_vec.push(
            VmixFunction::OverlayInput4(self.get_mov())
                .to_string()
                .into(),
        );
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
        VmixFunction::SetPanX(VmixInfo {
            id: &self.get_mov(),
            value: pan.to_string(),
            prop: VmixProperty::Mov(self.get_mov()),
        })
        .to_string()
        .into()
    }

    fn play_anim(&mut self) -> Vec<JsString> {
        vec![
            VmixFunction::Restart(self.get_mov()).to_string().into(),
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
        //log(&format!("round_ind pre {}", round_ind));
        self.round_ind = round_ind;
        let t = self.reset_scores();
        //log(&format!("round_ind post {}", self.round_ind));
        t
    }

    // LB TCP
    fn set_lb_pos(&mut self) -> JsString {
        let thing: JsString = VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: self.position.to_string(),
            prop: VmixProperty::LBPosition(self.position, self.lb_pos, self.lb_even),
        })
        .to_string()
        .into();
        log(&String::from(thing.clone()));
        thing
    }

    fn set_lb_name(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: self.name.clone(),
            prop: VmixProperty::LBName(self.position),
        })
        .to_string()
        .into()
    }

    fn set_lb_hr(&self) -> JsString {
        let value = if self.hot_round && self.round_ind != 0 {
            r"x:\FLIPUP\grafik\fire.png"
        } else {
            r"x:\FLIPUP\grafik\alpha.png"
        };
        VmixFunction::SetImage(VmixInfo {
            id: &self.lb_vmix_id,
            value: value.to_string(),
            prop: VmixProperty::LBHotRound(self.position),
        })
        .to_string()
        .into()
    }

    fn set_rs(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: self.round_score.to_string(),
            prop: VmixProperty::LBRS(self.position),
        })
        .to_string()
        .into()
    }

    fn set_ts(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: self.total_score.to_string(),
            prop: VmixProperty::LBTS(self.position),
        })
        .to_string()
        .into()
    }

    fn set_moves(&self) -> Vec<JsString> {
        self.rank.get_tcps(self.position, &self.lb_vmix_id)
    }

    fn set_thru(&self) -> JsString {
        VmixFunction::SetText(VmixInfo {
            id: &self.lb_vmix_id,
            value: (self.hole + 1).to_string(),
            prop: VmixProperty::LBThru(self.position),
        })
        .to_string()
        .into()
    }

    pub fn set_lb(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        return_vec.push(self.set_lb_pos());
        return_vec.push(self.set_lb_name());
        return_vec.push(self.set_lb_hr());
        return_vec.push(self.set_rs());
        return_vec.push(self.set_ts());
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
