use super::queries;
use crate::api::{Error, HoleUpdate};
use crate::controller::queries::layout::hole::Hole;
use crate::controller::queries::layout::Holes;
use crate::flipup_vmix_controls::LeaderBoardProperty;
use crate::flipup_vmix_controls::{OverarchingScore, Score};
use crate::vmix::functions::*;
use crate::{controller, dto};
use cynic::GraphQlResponse;
use itertools::Itertools;
use log::warn;
use rayon::prelude::*;
use rocket::futures::{FutureExt, StreamExt};
use rocket::State;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::RwLock;

pub const DEFAULT_FOREGROUND_COL: &str = "3F334D";
pub const DEFAULT_FOREGROUND_COL_ALPHA: &str = "3F334D00";
pub const DEFAULT_BACKGROUND_COL: &str = "574B60";

#[derive(Debug, Clone, Default)]
pub enum RankUpDown {
    Up(i16),
    Down(i16),
    #[default]
    Same,
}

pub trait HoleScoreOrDefault {
    fn hole_score(&self, hole: usize) -> isize;
    fn score_to_hole(&self, hole: usize) -> isize;
    fn get_hole_info(&self, hole: u8) -> Vec<VMixFunction<VMixProperty>>;
}

impl HoleScoreOrDefault for Option<&PlayerRound> {
    fn hole_score(&self, hole: usize) -> isize {
        match self {
            Some(round) => round.hole_score(hole),
            None => isize::MAX,
        }
    }
    fn score_to_hole(&self, hole: usize) -> isize {
        match self {
            Some(round) => round.score_to_hole(hole),
            None => isize::MAX,
        }
    }
    fn get_hole_info(&self, hole: u8) -> Vec<VMixFunction<VMixProperty>> {
        match self {
            Some(round) => round.get_hole_info(hole),
            None => vec![],
        }
    }
}

impl RankUpDown {
    fn get_instructions(&self, pos: usize) -> [VMixFunction<LeaderBoardProperty>; 2] {
        [self.make_move(pos), self.make_arrow(pos)]
    }

    fn make_move(&self, pos: usize) -> VMixFunction<LeaderBoardProperty> {
        let movement = match self {
            RankUpDown::Up(val) => val.to_string(),
            RankUpDown::Down(val) => val.to_string(),
            RankUpDown::Same => "".to_string(),
        };

        VMixFunction::SetText {
            value: movement,
            input: LeaderBoardProperty::Move { pos }.into(),
        }
    }

    fn make_arrow(&self, pos: usize) -> VMixFunction<LeaderBoardProperty> {
        let img = match self {
            RankUpDown::Up(_) => r"x:\FLIPUP\grafik\greentri.png",
            RankUpDown::Down(_) => r"x:\FLIPUP\grafik\redtri.png",
            RankUpDown::Same => r"x:\FLIPUP\grafik\alpha.png",
        }
        .to_string();

        VMixFunction::SetImage {
            value: img,
            input: LeaderBoardProperty::Arrow { pos }.into(),
        }
    }
}
// TODO: Refactor out
#[derive(Debug, Clone, Default)]
pub struct PlayerRound {
    results: Vec<HoleResult>,
    finished: bool,
    round: usize,
}

#[derive(Debug, Clone)]
pub struct HoleResult {
    pub hole: u8,
    pub throws: u8,
    pub hole_representation: Arc<Hole>,
    tjing_result: Option<queries::HoleResult>,
    ob: HashSet<usize>,
    finished: bool,
}

impl From<&HoleResult> for Score {
    fn from(res: &HoleResult) -> Self {
        Self::new(
            res.throws(),
            res.hole_representation.par as i8,
            res.hole,
        )
    }
}

impl HoleResult {
    pub fn new(hole: u8, holes: &Holes) -> Result<Self,Error> {
        Ok(Self {
            hole,
            throws: 0,
            hole_representation: holes.find_hole(hole).ok_or(Error::UnableToParse)?,
            tjing_result: None,
            ob: HashSet::new(),
            finished: false,
        })
    }

    pub fn from_tjing(
        hole: u8,
        holes: &controller::queries::layout::hole::Holes,
        tjing: controller::queries::HoleResult,
    ) -> Option<Self> {
        Some(Self {
            hole,
            throws: tjing.score as u8,
            hole_representation: holes.find_hole(hole)?,
            tjing_result: Some(tjing),
            ob: HashSet::new(),
            finished: false,
        })
    }
    
    pub fn tjing_result(self) -> Option<queries::HoleResult> {
        self.tjing_result
    }
    

    
    
    
    pub fn actual_score(&self) -> i8 {
        self.throws() - self.hole_representation.par as i8
    }

    fn throws(&self) -> i8 {
        if let Some(tjing) = &self.tjing_result {
            tjing.score as i8
        } else {
            0 // TEMP
        }
    }

    fn to_score(&self) -> Score {
        self.into()
    }

    pub fn get_score_colour(&self, player: usize) -> VMixFunction<VMixProperty> {
        self.to_score().update_score_colour(player)
    }

    pub fn get_mov(&self, player: usize) -> [VMixFunction<VMixProperty>; 2] {
        self.to_score().play_mov_vmix(player, false)
    }
}

impl PlayerRound {
    fn new(results: Vec<HoleResult>, round: usize) -> Self {
        Self {
            results,
            finished: false,
            round,
        }
    }

    pub fn add_new_hole(&mut self, all_holes: &Holes) -> Result<(), Error> {
        if self.results.len() >= 18 {
            return Err(Error::TooManyHoles);
        }
        self.results.push(
            HoleResult::new(self.results.len() as u8, all_holes)?,
        );
        Ok(())
    }

    pub fn current_result_mut(&mut self, hole: usize) -> Option<&mut HoleResult> {
        for result in self.results.iter_mut() {
            if let Some(ref tjing_result) = result.tjing_result {
                if tjing_result.hole.number as usize == hole {
                    return Some(result);
                }
            }
        }
        None
    }
    
    pub fn tjing_results(self) -> Vec<Option<queries::HoleResult>> {
        self.results.into_iter().map(|res| res.tjing_result).collect()
    }
    
    pub fn update_tjing(&mut self, results: &[Option<queries::HoleResult>]) {
        for result in &mut self.results {
            if let Some(Some(tjing_result)) = results.get(result.hole as usize) {
                result.tjing_result = Some(tjing_result.to_owned());
            }
        }
    }
}

impl PlayerRound {
    pub fn current_result(&self, hole: u8) -> Option<&HoleResult> {
        self.results.iter().find(|result| result.hole == hole)
    }

    // Gets score up until hole
    pub fn score_to_hole(&self, hole: usize) -> isize {
        //log(&format!("hole {}", hole));
        (0..hole + 1).map(|i| self.hole_score(i)).sum()
    }

    fn hole_score(&self, hole: usize) -> isize {
        match self.results.get(hole) {
            Some(result) => result.actual_score() as isize,
            None => isize::MAX,
        }
    }

    pub fn get_hole_info(&self, hole: u8) -> Vec<VMixFunction<VMixProperty>> {
        let mut r_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        let hole = self.current_result(hole).unwrap();

        r_vec.push(VMixFunction::SetText {
            value: hole.hole.to_string(),
            input: VMixProperty::Hole.into(),
        });

        r_vec.push(VMixFunction::SetText {
            value: hole.hole_representation.par.to_string(),
            input: VMixProperty::HolePar.into(),
        });

        r_vec.push(VMixFunction::SetText {
            value: (hole.hole_representation.length as u64).to_string() + "M",
            input: VMixProperty::HoleMeters.into(),
        });

        let feet = (hole.hole_representation.length as f64 * 3.28084) as u64;
        r_vec.push(VMixFunction::SetText {
            value: feet.to_string() + "FT",
            input: VMixProperty::HoleFeet.into(),
        });
        r_vec
    }
}

pub fn fix_score(score: isize) -> String {
    use std::cmp::Ordering;

    match score.cmp(&0) {
        Ordering::Less => format!("{}", score),
        Ordering::Equal => "E".to_string(),
        Ordering::Greater => format!("%2B{}", score),
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub player_id: String,
    pub name: String,
    first_name: String,
    surname: String,
    pub rank: RankUpDown,
    pub image_url: Option<String>,
    pub total_score: isize,
    pub round_score: isize,
    round_ind: usize,
    pub results: PlayerRound,
    pub hole_shown_up_until: usize,
    pub ind: usize,
    pub index: usize,
    pub throws: u8,
    pub ob: bool,
    pub position: usize,
    pub lb_even: bool,
    pub hot_round: bool,
    pub lb_pos: usize,
    pub old_pos: usize,
    pos_visible: bool,
    pub lb_shown: bool,
    pub dnf: bool,
    pub first_scored: bool,
    pub thru: u8,
    pub visible_player: bool,
}

impl Player {
    fn from_query(player: queries::Player, round: usize, holes: &Holes) -> Self {
        let first_name = player.user.first_name.unwrap();
        let surname = player.user.last_name.unwrap();
        let image_id: Option<String> = player
            .user
            .profile
            .and_then(|profile| profile.profile_image_url);
        let results = PlayerRound::new(
            player
                .results
                .unwrap_or_default()
                .into_iter()
                .map(|r: controller::queries::HoleResult| {
                    HoleResult::from_tjing(r.hole.number as u8, holes, r)
                        .expect("Could not create HoleResult")
                })
                .collect_vec(),
            round,
        );
        Self {
            player_id: player.id.into_inner(),
            image_url: image_id,
            results,
            first_name: first_name.clone(),
            surname: surname.clone(),
            rank: Default::default(),
            total_score: 0,
            name: format!("{} {}", first_name, surname),
            dnf: player.dnf.is_dnf || player.dns.is_dns,
            first_scored: false,
            round_ind: round,
            thru: 0,
            index: 0,
            hot_round: false,
            hole_shown_up_until: 0,
            ind: 0,
            throws: 0,
            ob: false,
            position: 0,
            lb_pos: 0,
            old_pos: 0,
            pos_visible: false,
            round_score: 0,
            lb_even: false,
            lb_shown: false,
            visible_player: false,
        }
    }
    pub fn null_player() -> Self {
        Player {
            player_id: "".to_string(),
            name: "".to_string(),
            first_name: "".to_string(),
            surname: "".to_string(),
            rank: Default::default(),
            total_score: 0,
            round_score: 0,
            round_ind: 0,
            index: 0,
            results: Default::default(),
            image_url: None,
            hole_shown_up_until: 0,
            ind: 0,
            throws: 0,
            ob: false,
            position: 0,
            lb_even: false,
            hot_round: false,
            lb_pos: 0,
            old_pos: 0,
            pos_visible: false,
            lb_shown: false,
            dnf: false,
            first_scored: false,
            thru: 0,
            visible_player: false,
        }
    }
}
impl From<&Player> for crate::flipup_vmix_controls::OverarchingScore {
    fn from(player: &Player) -> Self {
        Self::new(
            player.round_ind,
            player.round_score,
            player.ind,
            player.total_score,
        )
    }
}

impl Player {
    pub fn get_round_total_score(&self) -> isize {
        self.round_score
    }

    pub fn score_before_round(&mut self) -> isize {
        self.total_score - self.round_score
    }

    pub fn get_current_shown_score(&self) -> Result<Score, Error> {
        self.results
            .results
            .iter()
            .find(|result| result.hole as usize == (self.hole_shown_up_until + 1))
            .ok_or(Error::NoScoreFound {
                player: self.name.clone(),
                hole: self.hole_shown_up_until + 1,
            })
            .map(Score::from)
    }

    pub fn get_score(&self, hole: usize) -> Result<Score, Error> {
        self.results
            .results
            .iter()
            .find(|result| result.hole as usize == hole + 1)
            .ok_or(Error::NoScoreFound {
                player: self.name.clone(),
                hole,
            })
            .map(Score::from)
    }

    pub fn check_if_allowed_to_visible(&mut self) {
        if self.dnf {
            self.lb_shown = false
        }
    }

    // Below goes JS TCP Strings

    pub fn set_name(&self) -> Vec<VMixFunction<VMixProperty>> {
        vec![
            VMixFunction::SetText {
                value: self.first_name.clone(),
                input: VMixProperty::Name(self.ind).into(),
            },
            VMixFunction::SetText {
                value: self.surname.clone(),
                input: VMixProperty::Surname(self.ind).into(),
            },
        ]
    }

    pub fn amount_of_holes_finished(&self) -> usize {
        self.results
            .results
            .iter()
            .filter(|res| res.finished)
            .count()
    }
    fn overarching_score_representation(&self) -> OverarchingScore {
        OverarchingScore::from(self)
    }

    pub fn set_all_values(&self) -> Result<Vec<VMixFunction<VMixProperty>>, Error> {
        let mut return_vec = vec![];
        return_vec.extend(self.set_name());
        if let Some(set_pos) = self.set_pos() {
            return_vec.push(set_pos);
        }
        return_vec.push(self.set_tot_score());
        if self.hole_shown_up_until != 0 {
            let funcs: Vec<_> = (0..self.hole_shown_up_until)
                .par_bridge()
                .flat_map(|hole| self.get_score(hole).unwrap().update_score(1))
                .collect();
            return_vec.extend(funcs);
        }
        return_vec.extend(self.delete_all_scores_after_current());

        return_vec.push(self.set_throw());
        Ok(return_vec)
    }

    pub fn increase_score(&mut self) -> Result<Vec<VMixFunction<VMixProperty>>, Error> {
        let mut return_vec: Vec<VMixFunction<VMixProperty>> = vec![];

        if !self.first_scored {
            self.first_scored = true;
        }


        let s = match self.get_score(self.hole_shown_up_until) {
            Ok(s) => s,
            Err(Error::NoScoreFound {..}) => {
                let Some(t) = self.results.current_result_mut(self.hole_shown_up_until) else {
                    return Err(Error::NoScoreFound {
                        player: self.name.clone(),
                        hole: self.hole_shown_up_until,
                    })
                };
                t.throws = self.throws;
                t.finished = true;
                t.to_score()
            }
            Err(e) => return Err(e),
        };
        // Update score text, visibility, and colour

        let score = self.get_current_shown_score()?.update_score(1);


        self.round_score += s.par_score() as isize;
        self.total_score += dbg!(s.par_score()) as isize;


        return_vec.extend(score);

        let overarching = self.overarching_score_representation();

        return_vec.push(self.set_tot_score());
        return_vec.extend(overarching.set_round_score());
        self.hole_shown_up_until += 1;
        self.throws = 0;
        return_vec.push(self.set_throw());
        Ok(return_vec)
    }
    fn add_total_score(&self, outside_instructions: &mut Vec<VMixFunction<VMixProperty>>) {
        outside_instructions.push(self.overarching_score_representation().set_total_score())
    }

    fn add_round_score(&self, outside_instructions: &mut Vec<VMixFunction<VMixProperty>>) {
        outside_instructions.extend(self.overarching_score_representation().set_round_score())
    }
    pub fn revert_hole_score(&mut self) -> Vec<VMixFunction<VMixProperty>> {
        let mut return_vec = vec![];
        if self.hole_shown_up_until > 0 {
            self.hole_shown_up_until -= 1;
            return_vec.extend(self.del_current_score());
            let result = self.results.hole_score(self.hole_shown_up_until);
            self.round_score -= result;
            self.total_score -= result;
            // Previously had shift-scores here
            return_vec.push(self.set_tot_score());
            self.add_round_score(&mut return_vec);
        }
        return_vec
    }

    fn set_tot_score(&self) -> VMixFunction<VMixProperty> {
        VMixFunction::SetText {
            value: fix_score(self.total_score),
            input: VMixProperty::TotalScore(self.ind).into(),
        }
    }

    pub fn set_pos(&self) -> Option<VMixFunction<VMixProperty>> {
        let value_string = if self.lb_even {
            "T".to_string()
        } else {
            "".to_string()
        } + &self.lb_pos.to_string();

        if self.visible_player {
            Some(VMixFunction::SetText {
                value: value_string,
                input: VMixProperty::PlayerPosition(self.ind as u16).into(),
            })
        } else {
            None
        }
    }

    pub fn hide_pos(&mut self) -> [VMixFunction<VMixProperty>; 3] {
        self.pos_visible = false;
        [
            VMixFunction::SetTextVisibleOff {
                input: VMixProperty::PlayerPosition(self.ind as u16).into(),
            },
            VMixFunction::SetColor {
                color: "00000000",
                input: VMixProperty::PosRightTriColor(self.ind).into(),
            },
            VMixFunction::SetColor {
                color: "00000000",
                input: VMixProperty::PosSquareColor(self.ind).into(),
            },
        ]
    }

    pub fn show_pos(&mut self) -> [VMixFunction<VMixProperty>; 3] {
        self.pos_visible = true;
        [
            VMixFunction::SetTextVisibleOn {
                input: VMixProperty::PlayerPosition(self.ind as u16).into(),
            },
            VMixFunction::SetColor {
                color: DEFAULT_BACKGROUND_COL,
                input: VMixProperty::PosRightTriColor(self.ind).into(),
            },
            VMixFunction::SetColor {
                color: DEFAULT_BACKGROUND_COL,
                input: VMixProperty::PosSquareColor(self.ind).into(),
            },
        ]
    }

    pub fn toggle_pos(&mut self) -> [VMixFunction<VMixProperty>; 3] {
        if self.pos_visible {
            self.hide_pos()
        } else {
            self.show_pos()
        }
    }

    /*pub fn shift_scores(&mut self, last_blank: bool) -> Vec<VMixFunction<VMixProperty>> {
        let mut return_vec = vec![];
        let in_hole = self.hole_shown_up_until;

        let diff = self.hole_shown_up_until - 8 + {
            if last_blank && self.hole_shown_up_until != 17 {
                1
            } else {
                0
            }
        };

        self.hole_shown_up_until = diff;
        self.shift = diff;
        for _ in diff..=in_hole {
            return_vec.extend(self.set_hole_score());
        }
        if last_blank && self.hole_shown_up_until != 18 {
            return_vec.push(VMixFunction::SetText {
                value: (in_hole + 2).to_string(),
                input: VMixProperty::HoleNumber(9, self.ind).into(),
            });
            return_vec.push(VMixFunction::SetTextVisibleOff {
                input: VMixProperty::Score {
                    hole: 9,
                    player: self.ind,
                }
                .into(),
            });
            return_vec.push(VMixFunction::SetColor {
                color: DEFAULT_FOREGROUND_COL_ALPHA,
                input: VMixProperty::ScoreColor {
                    hole: 9,
                    player: self.ind,
                }
                .into(),
            });
        }
        if self.visible_player {
            return_vec
        } else {
            vec![]
        }
    }*/

    fn del_score(&self, hole: usize) -> [VMixFunction<VMixProperty>; 4] {
        let score_prop = VMixProperty::Score {
            hole,
            player: self.ind,
        };

        let col_prop = VMixProperty::ScoreColor {
            hole,
            player: self.ind,
        };
        let h_num_prop = VMixProperty::HoleNumber(self.hole_shown_up_until + 1, self.ind);
        [
            VMixFunction::SetText {
                value: "".to_string(),
                input: score_prop.clone().into(),
            },
            VMixFunction::SetColor {
                color: DEFAULT_FOREGROUND_COL_ALPHA,
                input: col_prop.into(),
            },
            VMixFunction::SetText {
                value: hole.to_string(),
                input: h_num_prop.into(),
            },
            VMixFunction::SetTextVisibleOff {
                input: score_prop.into(),
            },
        ]
    }

    fn del_current_score(&self) -> [VMixFunction<VMixProperty>; 4] {
        self.del_score(self.hole_shown_up_until + 1)
    }

    fn delete_all_scores_after_current(&self) -> Vec<VMixFunction<VMixProperty>> {
        ((self.hole_shown_up_until + 1)..=18)
            .par_bridge()
            .flat_map(|hole| self.del_score(hole))
            .collect()
    }

    pub fn reset_scores(&mut self) -> Vec<VMixFunction<VMixProperty>> {
        let mut return_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        return_vec.extend(self.delete_all_scores_after_current());
        self.hole_shown_up_until = 0;
        self.round_score = 0;
        self.total_score = self.score_before_round();

        self.add_total_score(&mut return_vec);
        return_vec.extend(self.hide_pos());
        self.add_round_score(&mut return_vec);
        return_vec
    }

    pub fn set_throw(&self) -> VMixFunction<VMixProperty> {
        VMixFunction::SetText {
            value: self.throws.to_string(),
            input: VMixProperty::Throw(self.ind).into(),
        }
    }

    pub fn set_round(&mut self, round_ind: usize) -> Vec<VMixFunction<VMixProperty>> {
        self.round_ind = round_ind;
        self.reset_scores()
    }

    fn set_lb_pos(&mut self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.position.to_string(),
            input: LeaderBoardProperty::Position {
                pos: self.position,
                lb_pos: self.lb_pos,
                tied: self.lb_even,
            }
            .into(),
        }
    }

    fn set_lb_name(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.name.clone(),
            input: LeaderBoardProperty::Name(self.position).into(),
        }
    }

    fn set_lb_hr(&self) -> VMixFunction<LeaderBoardProperty> {
        let value = if self.hot_round
            && self.round_ind != 0
            && self.hole_shown_up_until != 0
            && self.hole_shown_up_until < 19
        {
            r"X:\FLIPUP\grafik\fire.png"
        } else {
            r"X:\FLIPUP\grafik\alpha.png"
        };
        VMixFunction::SetImage {
            value: value.to_string(),
            input: LeaderBoardProperty::HotRound(self.position).into(),
        }
    }

    fn set_rs(&self, hidden: bool) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: if self.lb_pos == 0 {
                "".to_string()
            } else if hidden {
                "E".to_string()
            } else {
                fix_score(self.round_score)
            },
            input: LeaderBoardProperty::RoundScore(self.position).into(),
        }
    }

    fn set_ts(&self, hidden: bool) -> [VMixFunction<LeaderBoardProperty>; 3] {
        [
            VMixFunction::SetTextVisibleOn {
                input: LeaderBoardProperty::TotalScore { pos: self.position }.into(),
            },
            VMixFunction::SetTextVisibleOn {
                input: LeaderBoardProperty::TotalScoreTitle.into(),
            },
            VMixFunction::SetText {
                value: if self.lb_pos == 0 {
                    "".to_string()
                } else if hidden && self.round_ind == 0 {
                    "E".to_string()
                } else {
                    fix_score(self.total_score)
                },
                input: LeaderBoardProperty::TotalScore { pos: self.position }.into(),
            },
        ]
    }

    fn set_moves(&self) -> [VMixFunction<LeaderBoardProperty>; 2] {
        self.rank.get_instructions(self.position)
    }

    fn set_thru(&self, hidden: bool) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: if self.lb_pos == 0 {
                "".to_string()
            } else if hidden {
                "0".to_string()
            } else {
                self.thru.to_string()
            },
            input: LeaderBoardProperty::Thru(self.position).into(),
        }
    }

    pub fn set_lb(&mut self) -> Vec<VMixFunction<LeaderBoardProperty>> {
        self.check_if_allowed_to_visible();
        let hide = self.thru == 0;
        let mut return_vec = vec![
            self.set_lb_pos(),
            self.set_lb_name(),
            self.set_lb_hr(),
            self.set_rs(hide),
        ];
        return_vec.extend(self.set_ts(hide));
        return_vec.extend(self.set_moves());
        return_vec.push(self.set_thru(hide));
        return_vec
    }
}

#[derive(Clone, Debug)]
pub struct RustHandler {
    pub chosen_division: cynic::Id,
    round_ids: Vec<String>,
    player_container: PlayerContainer,
    divisions: Vec<queries::Division>,
    round_ind: usize,
    pub groups: Vec<Vec<dto::Group>>,
}

#[derive(Clone, Debug)]
struct PlayerContainer {
    rounds_with_players: Vec<Vec<Player>>,
    round: usize,
}
impl PlayerContainer {
    fn new(rounds_with_players: Vec<Vec<Player>>) -> Self {
        Self {
            rounds_with_players,
            round: 0,
        }
    }

    pub fn set_round(&mut self, round: usize) -> Result<(), &'static str> {
        if self.rounds_with_players.len() > round {
            Err("That round does not exist")
        } else {
            self.round = round;
            Ok(())
        }
    }

    pub fn players(&self) -> &Vec<Player> {
        self.rounds_with_players.get(self.round).unwrap()
    }
    
    pub fn players_mut(&mut self) -> &mut Vec<Player> {
        self.rounds_with_players.get_mut(self.round).unwrap()
    }
}

impl RustHandler {
    pub async fn new(event_id: &str) -> Result<Self, Error> {
        let time = std::time::Instant::now();
        let round_ids = Self::get_rounds(event_id).await?;
        let event = Self::get_event(event_id, round_ids.clone()).await;
        let groups = Self::get_groups(event_id).await;
        warn!("Time taken to get event: {:?}", time.elapsed());
        let mut divisions: Vec<queries::Division> = vec![];

        let holes = dbg!(Self::get_holes(event_id).await?);
        event
            .iter()
            .flat_map(|round| &round.event)
            .flat_map(|event| event.divisions.clone())
            .flatten()
            .unique_by(|division| division.id.clone())
            .for_each(|division| divisions.push(division));

        let mut container = PlayerContainer::new(
            event
                .into_iter()
                .enumerate()
                .flat_map(|(round_num, round)| Some((round_num, round.event?)))
                .map(|(round_num, event)| {
                    let holes = holes.get(round_num).expect("hole should exist");
                    event
                        .players
                        .into_iter()
                        .map(|player| Player::from_query(player, round_num,holes))
                        .collect_vec()
                })
                .collect_vec(),
        );

        for (i, round) in container.rounds_with_players.iter_mut().enumerate() {
            round.iter_mut().for_each(|player| {
                if let Some(group_index) = groups[i]
                    .iter()
                    .flat_map(|group| &group.players)
                    .enumerate()
                    .find(|(_, group_player)| group_player.id == player.player_id)
                    .map(|(group_index, _)| group_index)
                {
                    player.index = group_index;
                }
            });
        }

        Ok(Self {
            chosen_division: divisions.first().expect("NO DIV CHOSEN").id.clone(),
            round_ids,
            player_container: container,
            divisions,
            groups,
            round_ind: 0,
        })
    }

    pub async fn get_event(
        event_id: &str,
        round_ids: Vec<String>,
    ) -> Vec<queries::RoundResultsQuery> {
        use cynic::QueryBuilder;
        use queries::*;
        let mut rounds = vec![];
        for id in round_ids {
            let operation = RoundResultsQuery::build(RoundResultsQueryVariables {
                event_id: event_id.into(),
                round_id: id.to_owned().into(),
            });
            let response = reqwest::Client::new()
                .post("https://api.tjing.se/graphql")
                .json(&operation)
                .send()
                .await
                .expect("failed to send request");

            let out = response
                .json::<GraphQlResponse<queries::RoundResultsQuery>>()
                .await
                .expect("failed to parse response")
                .data
                .unwrap();
            rounds.push(out);
        }
        rounds
    }

    pub async fn get_rounds(event_id: &str) -> Result<Vec<String>, Error> {
        use cynic::QueryBuilder;
        use queries::round::{RoundsQuery, RoundsQueryVariables};
        let body = RoundsQuery::build(RoundsQueryVariables {
            event_id: event_id.into(),
        });
        let response = reqwest::Client::new()
            .post("https://api.tjing.se/graphql")
            .json(&body)
            .send()
            .await
            .map_err(|_| Error::UnableToParse)?
            .json::<GraphQlResponse<RoundsQuery>>()
            .await;
        Ok(response
            .unwrap()
            .data
            .unwrap()
            .event
            .unwrap()
            .rounds
            .into_iter()
            .flatten()
            .map(|round| round.id.into_inner())
            .collect_vec())
    }

    pub async fn get_holes(event_id: &str) -> Result<Vec<Holes>, Error> {
        use cynic::QueryBuilder;
        use queries::layout::{HoleLayoutQuery, HoleLayoutQueryVariables};
        let body = HoleLayoutQuery::build(HoleLayoutQueryVariables {
            event_id: event_id.into(),
        });

        let holes = reqwest::Client::new()
            .post("https://api.tjing.se/graphql")
            .json(&body)
            .send()
            .await
            .unwrap()
            .json::<GraphQlResponse<HoleLayoutQuery>>()
            .await
            .unwrap();
        let holes = {
            let Some(data) = holes.data else {
                return Err(Error::UnableToParse)
            };
            let Some(event) = data.event else {
                return Err(Error::UnableToParse)
            };
            let mut rounds_holes = vec![];
            for round in event.rounds {
                let Some(round) = round else {
                    return Err(Error::UnableToParse)
                };
                let holes = round.pools.into_iter().flat_map(|pool| pool.layout_version.holes).dedup_by(|a,b| a.number == b.number).collect_vec();
                match Holes::try_from(holes) {
                    Err(e) => return Err(e),
                    Ok(holes) => rounds_holes.push(holes)
                }
            }
            rounds_holes
        };

        Ok(holes)
    }

    pub fn groups(&self) -> &Vec<dto::Group> {
        self.groups.get(self.round_ind).unwrap()
    }

    pub fn get_divisions(&self) -> Vec<queries::Division> {
        let mut divs: Vec<queries::Division> = vec![];
        for div in &self.divisions {
            divs.push(div.clone());
        }
        divs
    }

    async fn get_groups(event_id: &str) -> Vec<Vec<dto::Group>> {
        use cynic::QueryBuilder;
        use queries::group::{GroupsQuery, GroupsQueryVariables};
        let body = GroupsQuery::build(GroupsQueryVariables {
            event_id: event_id.into(),
        });

        let groups = reqwest::Client::new()
            .post("https://api.tjing.se/graphql")
            .json(&body)
            .send()
            .await
            .unwrap()
            .json::<GraphQlResponse<GroupsQuery>>()
            .await
            .unwrap();

        groups
            .data
            .unwrap()
            .event
            .unwrap()
            .rounds
            .into_iter()
            .flatten()
            .map(|round: queries::group::Round| {
                round
                    .pools
                    .into_iter()
                    .flat_map(|pool| pool.groups)
                    .dedup_by(|group1, group2| group1.id == group2.id)
                    .map(dto::Group::from)
                    .collect_vec()
            })
            .collect_vec()
    }
    
    pub fn round_id(&self) -> &str {
        &self.round_ids[self.round_ind]
    }

    pub fn amount_of_rounds(&self) -> usize {
        self.player_container.rounds_with_players.len()
    }

    pub fn set_chosen_by_ind(&mut self, ind: usize) {
        self.chosen_division = self.divisions[ind].id.clone();
    }

    pub fn find_player_mut(&mut self, player_id: String) -> Option<&mut Player> {
        self.player_container
            .players_mut()
            .into_iter()
            .find(|player| player.player_id == player_id)
    }

    pub fn get_players(&self) -> Vec<&Player> {
        self.player_container.players().iter().collect_vec()
    }

    pub fn get_players_mut(&mut self) -> Vec<&mut Player> {
        self.player_container
            .players_mut()
            .into_iter()
            .collect_vec()
    }
}
