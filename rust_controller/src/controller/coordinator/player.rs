use std::sync::Arc;

use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use rayon::prelude::*;

use crate::api::Error;
use crate::controller::get_data::{HoleResult, DEFAULT_FOREGROUND_COL_ALPHA};
use crate::controller::queries;
use crate::controller::queries::layout::Holes;
use crate::controller::queries::Division;
use crate::flipup_vmix_controls::{
    Image, LeaderBoardProperty, Leaderboard, LeaderboardMovement, OverarchingScore, Score,
};
use crate::vmix::functions::{
    Compare2x2, CurrentPlayer, VMixHoleInfo, VMixInterfacer, VMixPlayerInfo,
};
use crate::{controller, util};

// TODO: Refactor out
#[derive(Debug, Clone, Default)]
pub struct PlayerRound {
    results: Vec<HoleResult>,
    start_at_hole: u8,
    finished: bool,
    round: usize,
}

impl PlayerRound {
    pub fn new(mut results: Vec<HoleResult>, round: usize, start_at_hole: u8) -> Self {
        results.sort_by(|a, b| a.hole.cmp(&b.hole));
        Self {
            results,
            finished: false,
            round,
            start_at_hole,
        }
    }

    pub fn add_new_hole(&mut self, all_holes: &Holes) -> Result<(), Error> {
        if self.results.len() >= 18 {
            return Err(Error::TooManyHoles);
        }
        self.results
            .push(HoleResult::new(self.results.len() as u8, all_holes)?);
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
        self.results
            .into_iter()
            .map(|res| res.tjing_result)
            .collect()
    }

    pub fn update_tjing(&mut self, results: &[queries::HoleResult]) {
        for result in &mut self.results {
            if let Some(tjing_result) = results
                .iter()
                .find(|hole| hole.hole.number as u8 == result.hole)
            {
                result.tjing_result = Some(tjing_result.to_owned());
                result.finished = true;
            }
        }
    }
    pub fn current_result(&self, hole: u8) -> Option<&HoleResult> {
        self.results
            .iter()
            .find(|result| result.hole == if hole + 1 > 18 { 18 } else { hole + 1 })
    }

    // Gets score up until hole
    pub fn score_to_hole(&self, hole: usize) -> isize {
        (0..=hole).map(|i| self.hole_score(i)).sum()
    }

    pub(crate) fn hole_score(&self, hole: usize) -> isize {
        match self.results.get(hole) {
            Some(result) => result.actual_score() as isize,
            None => isize::MAX,
        }
    }

    pub fn get_hole_info(&self, hole: u8) -> Vec<VMixInterfacer<VMixHoleInfo>> {
        let mut r_vec: Vec<VMixInterfacer<VMixHoleInfo>> = vec![];
        let hole = self.current_result(hole).unwrap();

        r_vec.push(VMixInterfacer::set_hole_text(VMixHoleInfo::Hole(hole.hole)));

        r_vec.push(VMixInterfacer::set_hole_text(VMixHoleInfo::HolePar(
            hole.hole_representation.par,
        )));

        r_vec.push(VMixInterfacer::set_hole_text(VMixHoleInfo::HoleMeters(
            hole.hole_representation.length,
        )));

        let feet = (hole.hole_representation.length as f32 * 3.28084) as u16;
        r_vec.push(VMixInterfacer::set_hole_text(VMixHoleInfo::HoleFeet(feet)));
        r_vec
    }

    pub fn amount_of_holes_finished(&self) -> u8 {
        self.results
            .iter()
            .filter(|result| result.tjing_result.is_some() || result.finished)
            .count() as u8
    }

    pub fn the_latest_6_holes(self) -> Vec<HoleResult> {
        self.results
            .into_iter()
            .sorted_by_key(|result| result.hole)
            .rev()
            .take(6)
            .rev()
            .collect_vec()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Player {
    pub player_id: String,
    pub name: String,
    pub first_name: String,
    pub surname: String,
    pub rank: RankUpDown,
    pub image_url: Option<String>,
    pub total_score: isize,
    pub round_score: isize,
    pub round_ind: usize,
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
    pub(crate) pos_visible: bool,
    pub lb_shown: bool,
    pub dnf: bool,
    pub first_scored: bool,
    pub visible_player: bool,
    pub division: Arc<Division>,
    image_location: Option<String>,
}

impl Player {
    pub fn from_query(
        player: queries::Player,
        round: usize,
        holes: &Holes,
        divisions: Vec<Arc<Division>>,
        starts_at_hole: u8,
    ) -> Result<Self, Error> {
        let mut first_name = player.user.first_name.unwrap();
        let mut surname = player.user.last_name.unwrap();
        first_name.retain(char::is_alphabetic);
        surname.retain(char::is_alphabetic);
        let image_id: Option<String> = player
            .user
            .profile
            .and_then(|profile| profile.profile_image_url);
        let division = divisions
            .into_iter()
            .find(|div| div.id == player.division.id)
            .ok_or(Error::UnableToParse)?;
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
            starts_at_hole,
        );

        let image_location = image_id.clone().map(|image| {
            format!(
                "{}.{}",
                player.id.clone().into_inner(),
                image.split(".").last().unwrap_or_default()
            )
        });
        if let Some(image) = image_id.to_owned() {
            let img = image_location.clone().unwrap();
            std::thread::spawn(|| {
                let _ = util::download_image_to_file(image, img);
            });
        }

        Ok(Self {
            player_id: player.id.into_inner(),
            image_url: image_id,
            results,
            first_name: first_name.clone(),
            surname: surname.clone(),
            name: format!("{} {}", first_name, surname),
            dnf: player.dnf.is_dnf || player.dns.is_dns,
            round_ind: round,
            division,
            image_location,
            ..Default::default()
        })
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
            visible_player: false,
            ..Default::default()
        }
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

    pub fn set_name(&self) -> Vec<VMixInterfacer<VMixPlayerInfo>> {
        vec![
            VMixInterfacer::set_text(
                self.first_name.clone(),
                VMixPlayerInfo::Name(self.ind).into(),
            ),
            VMixInterfacer::set_text(
                self.surname.clone(),
                VMixPlayerInfo::Surname(self.ind).into(),
            ),
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

    pub fn set_all_values(
        &self,
        lb: &Leaderboard,
        max_all: bool,
    ) -> Result<Vec<VMixInterfacer<VMixPlayerInfo>>, Error> {
        let mut return_vec = vec![];
        return_vec.extend(self.set_name());
        if let Some(set_pos) = self.set_pos(lb) {
            return_vec.push(set_pos);
        }
        if max_all {
            let player_with_correct = lb.find_player_in_current_state(self);
            return_vec.push(player_with_correct.set_tot_score());
            return_vec.push(player_with_correct.set_round_score())
        } else {
            return_vec.push(self.set_tot_score());
            return_vec.push(self.set_round_score());
        }
        if max_all {
            for result in &self.results.results {
                return_vec.extend(result.to_score().update_score(0))
            }
        } else if self.hole_shown_up_until != 0 {
            let funcs: Vec<_> = (0..self.hole_shown_up_until)
                .par_bridge()
                .flat_map(|hole| self.get_score(hole).unwrap().update_score(0))
                .collect();
            return_vec.extend(funcs);
        }
        return_vec.extend(self.delete_all_scores_after_current(max_all));

        return_vec.push(self.set_throw());
        return_vec.extend(self.add_lb_things(lb));
        return_vec.extend(self.set_stats());
        Ok(return_vec)
    }

    pub fn set_all_compare_2x2_values(
        &self,
        index: usize,
        lb: &Leaderboard,
    ) -> Result<Vec<VMixInterfacer<Compare2x2>>, Error> {
        let mut output: Vec<VMixInterfacer<Compare2x2>> = self
            .set_all_values(lb, true)?
            .into_par_iter()
            .map(|val| val.into_compare_2x2_player(index))
            .collect();

        let img = if cfg!(target_os = "windows") {
            self.image_location.clone()
        } else {
            self.image_url.clone()
        };
        output.push(VMixInterfacer::set_image(
            img.unwrap_or_default(),
            Compare2x2::PlayerImage { index },
        ));
        Ok(output)
    }

    pub fn set_all_current_player_values(
        &self,
        interfaces: &[VMixInterfacer<VMixPlayerInfo>],
    ) -> Vec<VMixInterfacer<CurrentPlayer>> {
        let mut second_values: Vec<_> = interfaces
            .par_iter()
            .cloned()
            .flat_map(|interface| interface.into_current_player())
            .collect();
        second_values.extend(
            self.results
                .clone()
                .the_latest_6_holes()
                .par_iter()
                .enumerate()
                .flat_map(|(hole_index, result)| result.to_current_player(hole_index + 1))
                .collect::<Vec<_>>(),
        );
        second_values
    }

    /// Used by leaderboard    
    pub fn fix_round_score(&mut self, up_until: Option<u8>) {
        self.round_score = 0;
        for result in &self.results.results {
            self.round_score += result.actual_score() as isize;
        }
        self.total_score += self.round_score
    }

    pub fn increase_score(&mut self) -> Result<Vec<VMixInterfacer<VMixPlayerInfo>>, Error> {
        let mut return_vec: Vec<VMixInterfacer<VMixPlayerInfo>> = vec![];

        if !self.first_scored {
            self.first_scored = true;
        }

        let s = match self.get_score(self.hole_shown_up_until) {
            Ok(s) => s,
            Err(Error::NoScoreFound { .. }) => {
                let Some(t) = self.results.current_result_mut(self.hole_shown_up_until) else {
                    return Err(Error::NoScoreFound {
                        player: self.name.clone(),
                        hole: self.hole_shown_up_until,
                    });
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
        self.total_score += s.par_score() as isize;

        return_vec.extend(score);

        let overarching = self.overarching_score_representation();

        return_vec.push(self.set_tot_score());
        return_vec.extend(overarching.set_round_score());

        self.hole_shown_up_until += 1;
        self.throws = 0;
        return_vec.push(self.set_throw());
        Ok(return_vec)
    }

    pub fn add_lb_things(&self, lb: &Leaderboard) -> [VMixInterfacer<VMixPlayerInfo>; 3] {
        let lb_player = lb.get_lb_player(self);
        [
            VMixInterfacer::set_image(
                match lb_player.movement {
                    LeaderboardMovement::Up(_) => Image::RedTriDown,
                    LeaderboardMovement::Down(_) => Image::GreenTriUp,
                    LeaderboardMovement::Same => Image::Nothing,
                }
                .to_location(),
                VMixPlayerInfo::PositionArrow(0),
            ),
            VMixInterfacer::set_text(
                match lb_player.movement {
                    LeaderboardMovement::Up(n) | LeaderboardMovement::Down(n) => n.to_string(),
                    LeaderboardMovement::Same => " ".to_string(),
                },
                VMixPlayerInfo::PositionMove(0),
            ),
            VMixInterfacer::set_image(
                if lb_player.hot_round {
                    Image::Flames
                } else {
                    Image::Nothing
                }
                .to_location(),
                VMixPlayerInfo::HotRound(0),
            ),
        ]
    }
    fn add_total_score(&self, outside_instructions: &mut Vec<VMixInterfacer<VMixPlayerInfo>>) {
        outside_instructions.push(self.overarching_score_representation().set_total_score())
    }

    fn add_round_score(&self, outside_instructions: &mut Vec<VMixInterfacer<VMixPlayerInfo>>) {
        outside_instructions.extend(self.overarching_score_representation().set_round_score())
    }
    pub fn revert_hole_score(&mut self) -> Vec<VMixInterfacer<VMixPlayerInfo>> {
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

    fn set_tot_score(&self) -> VMixInterfacer<VMixPlayerInfo> {
        VMixInterfacer::set_text(
            controller::fix_score(self.total_score),
            VMixPlayerInfo::TotalScore(0).into(),
        )
    }

    fn set_round_score(&self) -> VMixInterfacer<VMixPlayerInfo> {
        VMixInterfacer::set_text(
            format!("({})", controller::fix_score(self.round_score)),
            VMixPlayerInfo::RoundScore(0),
        )
    }

    pub fn set_pos(&self, lb: &Leaderboard) -> Option<VMixInterfacer<VMixPlayerInfo>> {
        let lb_player = lb.get_lb_player(self);
        let value_string = if lb_player.tied.is_some() {
            "T".to_string()
        } else {
            "".to_string()
        } + &lb_player.position.to_string();

        Some(VMixInterfacer::set_text(
            value_string,
            VMixPlayerInfo::PlayerPosition(0),
        ))
    }

    /*pub fn shift_scores(&mut self, last_blank: bool) -> Vec<VMixInterfacer<VMixProperty>> {
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
            return_vec.push(VMixInterfacer::SetText {
                value: (in_hole + 2).to_string(),
                input: VMixProperty::HoleNumber(9, self.ind).into(),
            });
            return_vec.push(VMixInterfacer::SetTextVisibleOff {
                input: VMixProperty::Score {
                    hole: 9,
                    player: self.ind,
                }
                .into(),
            });
            return_vec.push(VMixInterfacer::SetColor {
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

    fn del_score(&self, hole: usize) -> [VMixInterfacer<VMixPlayerInfo>; 3] {
        let score_prop = VMixPlayerInfo::Score { hole, player: 0 };

        let col_prop = VMixPlayerInfo::ScoreColor { hole, player: 0 };
        [
            VMixInterfacer::set_text("".to_string(), score_prop.clone().into()),
            VMixInterfacer::set_color(DEFAULT_FOREGROUND_COL_ALPHA, col_prop.into()),
            VMixInterfacer::set_text_visible_off(score_prop.into()),
        ]
    }

    fn del_current_score(&self) -> [VMixInterfacer<VMixPlayerInfo>; 3] {
        self.del_score(self.hole_shown_up_until + 1)
    }

    fn delete_all_scores_after_current(
        &self,
        show_max: bool,
    ) -> Vec<VMixInterfacer<VMixPlayerInfo>> {
        if show_max {
            self.results
                .results
                .iter()
                .filter(|result| !result.finished && !result.tjing_result.is_some())
                .flat_map(|result| self.del_score(result.hole as usize))
                .collect_vec()
        } else {
            ((self.hole_shown_up_until + 1)..=18)
                .par_bridge()
                .flat_map(|hole| self.del_score(hole))
                .collect()
        }
    }

    pub fn reset_scores(&mut self) -> Vec<VMixInterfacer<VMixPlayerInfo>> {
        let mut return_vec: Vec<VMixInterfacer<VMixPlayerInfo>> = vec![];
        return_vec.extend(self.delete_all_scores_after_current(false));
        self.hole_shown_up_until = 0;
        self.round_score = 0;
        self.total_score = self.score_before_round();

        self.add_total_score(&mut return_vec);
        self.add_round_score(&mut return_vec);
        return_vec
    }

    pub fn set_throw(&self) -> VMixInterfacer<VMixPlayerInfo> {
        VMixInterfacer::set_color(
            &self.throws.to_string(),
            VMixPlayerInfo::Throw(self.ind).into(),
        )
    }

    fn set_lb_name(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            self.name.clone(),
            LeaderBoardProperty::Name(self.position).into(),
        )
    }

    fn set_lb_hr(&self) -> VMixInterfacer<LeaderBoardProperty> {
        let value = if self.hot_round
            && self.round_ind != 0
            && self.hole_shown_up_until != 0
            && self.hole_shown_up_until < 19
        {
            r"X:\FLIPUP\grafik\fire.png"
        } else {
            r"X:\FLIPUP\grafik\alpha.png"
        };
        VMixInterfacer::set_image(
            value.to_string(),
            LeaderBoardProperty::HotRound(self.position).into(),
        )
    }

    fn hide_rs(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            "".to_string(),
            LeaderBoardProperty::RoundScore(self.position).into(),
        )
    }

    fn set_ts(&self) -> [VMixInterfacer<LeaderBoardProperty>; 3] {
        [
            VMixInterfacer::set_text_visible_on(
                LeaderBoardProperty::TotalScore { pos: self.position }.into(),
            ),
            VMixInterfacer::set_text_visible_on(LeaderBoardProperty::TotalScoreTitle.into()),
            VMixInterfacer::set_text(
                controller::fix_score(self.total_score),
                LeaderBoardProperty::TotalScore { pos: self.position }.into(),
            ),
        ]
    }

    fn set_moves(&self) -> [VMixInterfacer<LeaderBoardProperty>; 2] {
        self.rank.get_instructions(self.position)
    }

    fn set_thru(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            if self.lb_pos == 0 {
                "".to_string()
            } else {
                self.hole_shown_up_until.to_string()
            },
            LeaderBoardProperty::Thru(self.position),
        )
    }

    // TODO: Remove this function.
    pub fn set_lb(&mut self) -> Vec<VMixInterfacer<LeaderBoardProperty>> {
        self.check_if_allowed_to_visible();
        let mut return_vec = vec![
            self.set_lb_name(),
            VMixInterfacer::set_image(
                Image::Nothing.to_location(),
                LeaderBoardProperty::HotRound(self.position),
            ),
            self.hide_rs(),
            VMixInterfacer::set_text(
                "".to_string(),
                LeaderBoardProperty::Position { pos: self.position },
            ),
            VMixInterfacer::set_text(
                "".to_string(),
                LeaderBoardProperty::TotalScore { pos: self.position },
            ),
        ];
        return_vec.extend(self.set_moves());
        return_vec.push(self.set_thru());
        return_vec
    }

    fn set_stats(&self) -> [VMixInterfacer<VMixPlayerInfo>; 2] {
        let results = self
            .results
            .results
            .iter()
            .filter_map(|result| result.tjing_result.as_ref())
            .collect_vec();

        let finished_holes = results.len() as f64;

        let (circle_hit, inside_putt) = if finished_holes > 0. {
            let circle_hit_count = results
                .par_iter()
                .filter(|result| result.is_circle_hit)
                .count() as f64;
            let circle_hit = (circle_hit_count / finished_holes * 100.).round() as usize;
            let inside_putt_count = results
                .par_iter()
                .filter(|result| result.is_inside_putt)
                .count() as f64;
            let inside_putt = (inside_putt_count / finished_holes * 100.).round() as usize;
            (circle_hit, inside_putt)
        } else {
            (0, 0)
        };
        [
            VMixInterfacer::set_text(format!("{circle_hit}%"), VMixPlayerInfo::CircleHit(0)),
            VMixInterfacer::set_text(format!("{inside_putt}%"), VMixPlayerInfo::InsidePutt(0)),
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub enum RankUpDown {
    Up(i16),
    Down(i16),
    #[default]
    Same,
}

impl RankUpDown {
    fn get_instructions(&self, pos: usize) -> [VMixInterfacer<LeaderBoardProperty>; 2] {
        [self.make_move(pos), self.make_arrow(pos)]
    }

    fn make_move(&self, pos: usize) -> VMixInterfacer<LeaderBoardProperty> {
        let movement = match self {
            RankUpDown::Up(val) => val.to_string(),
            RankUpDown::Down(val) => val.to_string(),
            RankUpDown::Same => "".to_string(),
        };

        VMixInterfacer::set_text(movement, LeaderBoardProperty::Move { pos })
    }

    fn make_arrow(&self, pos: usize) -> VMixInterfacer<LeaderBoardProperty> {
        let img = match self {
            RankUpDown::Up(_) => r"C:\livegrafik-flipup\greentri.png",
            RankUpDown::Down(_) => r"C:\livegrafik-flipup\redtri.png",
            RankUpDown::Same => r"C:\livegrafik-flipup\alpha.png",
        }
        .to_string();

        VMixInterfacer::set_image(img, LeaderBoardProperty::Arrow { pos }.into())
    }
}
