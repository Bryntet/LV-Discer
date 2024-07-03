use crate::flipup_vmix_controls::LeaderBoardProperty;
use cynic::GraphQlResponse;
use itertools::Itertools;
use log::warn;
use rayon::prelude::*;
use std::collections::HashMap;
use rocket::futures::StreamExt;
use crate::api::MyError;
use crate::controller::queries::group::GroupPlayerConnection;

use super::{queries};
use crate::controller::queries::HoleResult;
use crate::flipup_vmix_controls::{OverarchingScore, Score};
use crate::vmix::functions::*;
use crate::dto;
use crate::dto::Group;


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
    fn get_hole_info(&self, hole: usize) -> Vec<VMixFunction<VMixProperty>>;
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
    fn get_hole_info(&self, hole: usize) -> Vec<VMixFunction<VMixProperty>> {
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
    round: usize
}
impl PlayerRound {
    fn new(results: Vec<queries::HoleResult>, round: usize) -> Self {
        Self {
            results,
            finished: false,
            round

        }
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

    pub fn get_hole_info(&self, hole: usize) -> Vec<VMixFunction<VMixProperty>> {
        let mut r_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        let binding = self::queries::Hole::default();
        let hole = match &self.results.get(hole) {
            Some(hole) => &hole.hole,
            None => &binding,
        };
        r_vec.push(VMixFunction::SetText {
            value: hole.number.to_string(),
            input: VMixProperty::Hole.into(),
        });

        r_vec.push(VMixFunction::SetText {
            value: hole.par.expect("Par should always be set").to_string(),
            input: VMixProperty::HolePar.into(),
        });
        let meters = if hole.measure_in_meters.unwrap_or(false) {
            hole.length.unwrap_or(0.0)
        } else {
            hole.length.unwrap_or(0.0) * 0.9144
        };

        r_vec.push(VMixFunction::SetText {
            value: (meters as u64).to_string() + "M",
            input: VMixProperty::HoleMeters.into(),
        });

        let feet = (meters * 3.28084) as u64;
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
    pub total_score: isize,
    // Total score for all rounds
    pub round_score: isize,
    // Score for only current round
    round_ind: usize,
    pub results: PlayerRound,
    pub hole: usize,
    pub ind: usize,
    pub throws: u8,
    shift: usize,
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
    pub thru: usize,
    pub visible_player: bool,
    group_id: Option<String>
}

impl Player {
    pub fn null_player() -> Self {
        Player {
            player_id: "".to_string(),
            name: "".to_string(),
            first_name: "".to_string(),
            surname: "".to_string(),
            rank: Default::default(),
            group_id: None,
            total_score: 0,
            round_score: 0,
            round_ind: 0,
            results: Default::default(),
            hole: 0,
            ind: 0,
            throws: 0,
            shift: 0,
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

    fn from_query(player: queries::Player, round: usize) -> Self {
        let first_name = player.user.first_name.unwrap();
        let surname = player.user.last_name.unwrap();
        Self {
            player_id: player.user.id.unwrap().into_inner(),
            group_id: Some(player.results.clone().unwrap().first().unwrap().player_connection.group_id.clone().into_inner()),
            results: PlayerRound::new(player.results.unwrap_or_default(),round),
            first_name: first_name.clone(),
            surname: surname.clone(),
            rank: Default::default(),
            total_score: 0,
            name: format!("{} {}", first_name, surname),
            dnf: player.dnf.is_dnf || player.dns.is_dns,
            first_scored: false,
            round_ind: round,
            thru: 0,
            hot_round: false,
            hole: 0,
            ind: 0,
            throws: 0,
            shift: 0,
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
        let mut total_score = 0;
        self.total_score - self.round_score;

        // log(&format!("round_ind {} tot_score {}", self.round_ind, total_score));
        total_score
    }

    pub fn get_score(&self) -> Score {
        self.results.results.last().unwrap().into()
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

    fn overarching_score_representation(&self) -> OverarchingScore {
        OverarchingScore::from(self)
    }

    pub fn set_hole_score(&mut self) -> Vec<VMixFunction<VMixProperty>> {
        let mut return_vec: Vec<VMixFunction<VMixProperty>> = vec![];

        if !self.first_scored {
            self.first_scored = true;
        }

        // Update score text, visibility, and colour
        let score = self.get_score().update_score(self.ind);
        return_vec.extend(score);

        let overarching = self.overarching_score_representation();

        return_vec.push(self.set_tot_score());
        return_vec.extend(overarching.set_round_score());
        self.hole += 1;
        self.throws = 0;
        return_vec.push(self.set_throw());
        if self.visible_player {
            return_vec
        } else {
            vec![]
        }
    }
    fn add_total_score(&self, outside_instructions: &mut Vec<VMixFunction<VMixProperty>>) {
        outside_instructions.push(self.overarching_score_representation().set_total_score())
    }

    fn add_round_score(&self, outside_instructions: &mut Vec<VMixFunction<VMixProperty>>) {
        outside_instructions.extend(self.overarching_score_representation().set_round_score())
    }
    pub fn revert_hole_score(&mut self) -> Vec<VMixFunction<VMixProperty>> {
        let mut return_vec = vec![];
        if self.hole > 0 {
            self.hole -= 1;
            return_vec.extend(self.del_score());
            let result = self.results.hole_score(self.hole);
            self.round_score -= result;
            self.total_score -= result;
            if self.hole > 8 {
                self.hole -= 1;
                self.round_score -= result;
                self.total_score -= result;
                return_vec.extend(self.shift_scores(true));
            } else {
                return_vec.push(self.set_tot_score());
                self.add_round_score(&mut return_vec);
            }
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

    pub fn shift_scores(&mut self, last_blank: bool) -> Vec<VMixFunction<VMixProperty>> {
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
            return_vec.extend(self.set_hole_score());
        }
        if last_blank && self.hole != 18 {
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
    }

    fn del_score(&self) -> [VMixFunction<VMixProperty>; 4] {
        let score_prop = VMixProperty::Score {
            hole: self.hole + 1,
            player: self.ind,
        };

        let col_prop = VMixProperty::ScoreColor {
            hole: self.hole + 1,
            player: self.ind,
        };
        let h_num_prop = VMixProperty::HoleNumber(self.hole + 1, self.ind);
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
                value: (self.hole + 1).to_string(),
                input: h_num_prop.into(),
            },
            VMixFunction::SetTextVisibleOff {
                input: score_prop.into(),
            },
        ]
    }

    pub fn reset_scores(&mut self) -> Vec<VMixFunction<VMixProperty>> {
        let mut return_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        for i in 0..9 {
            self.hole = i;
            return_vec.extend(self.del_score());
        }
        self.hole = 0;
        self.shift = 0;
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

    // LB TCP
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
        let value = if self.hot_round && self.round_ind != 0 && self.hole != 0 && self.hole < 19 {
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
}

#[derive(Clone, Debug)]
struct PlayerContainer{
    rounds_with_players: Vec<Vec<Player>>,
    round: usize
}
impl PlayerContainer {
    fn new(rounds_with_players: Vec<Vec<Player>>) -> Self {
        Self {
            rounds_with_players,
            round: 0
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
}

impl RustHandler {
    pub async fn new(event_id: &str) -> Result<Self,MyError> {
        let time = std::time::Instant::now();
        let round_ids = Self::get_rounds(event_id).await?;
        let event= Self::get_event(event_id,round_ids.clone()).await;
        warn!("Time taken to get event: {:?}", time.elapsed());
        let mut divisions: Vec<queries::Division> = vec![];
        event
            .iter()
            .flat_map(|round|&round.event)
            .flat_map(|event|event.divisions.clone())
            .flatten()
            .unique_by(|division|division.id.clone())
            .for_each(|division|divisions.push(division));


        let container = PlayerContainer::new(event
            .into_iter()
            .enumerate()
            .flat_map(|(round_num,round)| Some((round_num, round.event?)))
            .map(|(round_num,event)|event.players.into_iter().map(|player|{
                
                Player::from_query(player,round_num)
            }).collect_vec())
            .collect_vec()
        );

        Ok(Self {
            chosen_division: divisions.first().expect("NO DIV CHOSEN").id.clone(),
            round_ids,
            player_container: container,
            divisions,
            round_ind: 0,
        })
    }



    pub async fn get_event(event_id: &str, round_ids: Vec<String>) -> Vec<queries::RoundResultsQuery> {
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
                .expect("failed to parse response").data.unwrap();
            rounds.push(out);
        }
        rounds

    }


    pub async fn get_rounds(event_id: &str) -> Result<Vec<String>, MyError> {
        use queries::round::{RoundsQuery,RoundsQueryVariables};
        use cynic::QueryBuilder;
        let body = RoundsQuery::build(RoundsQueryVariables {
            event_id: event_id.into()
        });
        let parse_error = MyError::UnableToParse("Unable to parse response");
        let response = reqwest::Client::new()
            .post("https://api.tjing.se/graphql")
            .json(&body)
            .send()
            .await
            .map_err(|_|parse_error)?
            .json::<GraphQlResponse<RoundsQuery>>()
            .await;
        Ok(response.unwrap().data.unwrap().event.unwrap().rounds.into_iter().flatten().map(|round|round.id.into_inner()).collect_vec())

    }

    pub fn get_divisions(&self) -> Vec<queries::Division> {
        let mut divs: Vec<queries::Division> = vec![];
        for div in &self.divisions {
            divs.push(div.clone());
        }
        divs
    }
    
    pub fn get_groups(&self) -> Vec<Vec<dto::Group>> {
        let mut groups: Vec<Vec<dto::Group>> = vec![];
        
        for round in &self.player_container.rounds_with_players {
            groups.push(vec![]);
            let mut map: HashMap<String,Vec<dto::Player>> = HashMap::new();
            round.iter()
                .flat_map(|player| Some((dto::Player::from(player), player.group_id.clone()?)))
                .for_each(|(dto_player,group_id)|map.entry(group_id).or_default().push(dto_player));
            for (id,players) in map.into_iter() {
                groups.last_mut().unwrap().push(Group::new(id,players))
            }
        }

        groups
    }

    pub fn get_players(self) -> Vec<Player> {
        self.player_container.players().clone()
    }
    pub fn set_chosen_by_ind(&mut self, ind: usize) {
        self.chosen_division = self.divisions[ind].id.clone();
    }
}
