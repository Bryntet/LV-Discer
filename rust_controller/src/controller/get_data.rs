use std::collections::HashSet;
use std::sync::Arc;

use cynic::GraphQlResponse;
use itertools::Itertools;
use log::warn;
use rayon::prelude::*;
use rocket::futures::{FutureExt, StreamExt};

use crate::api::Error;
use crate::controller::coordinator::player::{Player, PlayerRound};
use crate::controller::hole::{HoleStats, VMixHoleInfo};
use crate::controller::queries::layout::hole::Hole;
use crate::controller::queries::layout::Holes;
use crate::controller::queries::Division;
use crate::flipup_vmix_controls::{
    Image, LeaderBoardProperty, Leaderboard, LeaderboardMovement, LeaderboardState, LeaderboardTop6,
};
use crate::flipup_vmix_controls::{OverarchingScore, Score};
use crate::vmix::functions::*;
use crate::{controller, dto};

use super::queries;

pub const DEFAULT_FOREGROUND_COL: &str = "3F334D";
pub const DEFAULT_FOREGROUND_COL_ALPHA: &str = "3F334D00";
pub const DEFAULT_BACKGROUND_COL: &str = "574B60";

#[derive(Debug, Clone)]
pub struct HoleResult {
    pub hole: u8,
    pub throws: u8,
    pub hole_representation: Arc<Hole>,
    pub tjing_result: Option<queries::HoleResult>,
    pub ob: HashSet<usize>,
    pub finished: bool,
}

impl From<&HoleResult> for Score {
    fn from(res: &HoleResult) -> Self {
        Self::new(
            res.throws() as i8,
            res.hole_representation.par as i8,
            res.hole,
        )
    }
}

impl HoleResult {
    pub fn new(hole: u8, holes: &Holes) -> Result<Self, Error> {
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
        let hole_rep = holes.find_hole(hole)?;
        Some(Self {
            hole,
            throws: (tjing.score as i8 + hole_rep.par as i8) as u8,
            hole_representation: hole_rep,
            tjing_result: Some(tjing),
            ob: HashSet::new(),
            finished: false,
        })
    }

    pub fn tjing_result(self) -> Option<queries::HoleResult> {
        self.tjing_result
    }

    pub fn actual_score(&self) -> i8 {
        self.throws() as i8 - self.hole_representation.par as i8
    }

    fn throws(&self) -> u8 {
        if let Some(tjing) = &self.tjing_result {
            tjing.score as u8
        } else {
            self.throws
        }
    }

    pub(crate) fn to_score(&self) -> Score {
        self.into()
    }

    pub fn get_score_colour(&self, player: usize) -> VMixInterfacer<VMixPlayerInfo> {
        self.to_score().update_score_colour(player)
    }

    pub fn get_mov(&self, player: usize) -> [VMixInterfacer<VMixPlayerInfo>; 2] {
        self.to_score().play_mov_vmix(player, false)
    }

    pub fn to_leaderboard_top_6(
        &self,
        pos: usize,
        hole: usize,
    ) -> [VMixInterfacer<LeaderboardTop6>; 2] {
        [
            VMixInterfacer::set_text(
                fix_score(self.actual_score() as isize),
                LeaderboardTop6::LastScore { pos, hole },
            ),
            VMixInterfacer::set_color(
                self.to_score().get_score_colour(),
                LeaderboardTop6::LastScoreColour { pos, hole },
            ),
        ]
    }

    pub fn to_current_player(&self, hole: usize) -> [VMixInterfacer<CurrentPlayer>; 2] {
        [
            VMixInterfacer::set_text(
                fix_score(self.actual_score() as isize),
                CurrentPlayer(VMixPlayerInfo::Score { player: 0, hole }),
            ),
            VMixInterfacer::set_color(
                self.to_score().get_score_colour(),
                CurrentPlayer(VMixPlayerInfo::ScoreColor { player: 0, hole }),
            ),
        ]
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

#[derive(Clone, Debug)]
pub struct RustHandler {
    pub chosen_division: cynic::Id,
    round_ids: Vec<String>,
    player_container: PlayerContainer,
    divisions: Vec<Arc<queries::Division>>,
    round_ind: usize,
    pub groups: Vec<Vec<dto::Group>>,
}

#[derive(Clone, Debug)]
struct PlayerContainer {
    rounds_with_players: Vec<Vec<Player>>,
    round: usize,
}
impl PlayerContainer {
    fn new(rounds_with_players: Vec<Vec<Player>>, round: usize) -> Self {
        Self {
            rounds_with_players,
            round,
        }
    }

    pub fn players(&self) -> &Vec<Player> {
        self.rounds_with_players.get(self.round).unwrap()
    }

    pub fn previous_rounds_players(&self) -> Vec<&Player> {
        self.rounds_with_players
            .par_iter()
            .enumerate()
            .filter(|(round, _)| round < &self.round)
            .flat_map(|(_, player)| player)
            .collect()
    }

    pub fn players_mut(&mut self) -> Vec<&mut Player> {
        self.rounds_with_players
            .get_mut(self.round)
            .unwrap()
            .iter_mut()
            .collect_vec()
    }
}

impl RustHandler {
    pub async fn new(event_id: &str, round: usize) -> Result<Self, Error> {
        let time = std::time::Instant::now();
        let round_ids = Self::get_rounds(event_id).await?;
        let event = Self::get_event(event_id, round_ids.clone()).await;
        let groups = Self::get_groups(event_id).await;
        warn!("Time taken to get event: {:?}", time.elapsed());

        let holes = Self::get_holes(event_id).await?;
        let divisions = event
            .iter()
            .flat_map(|round| &round.event)
            .flat_map(|event| event.divisions.clone())
            .flatten()
            .dedup_by(|a, b| a.id == b.id)
            .map(Arc::new)
            .collect::<Vec<_>>();

        let mut container = PlayerContainer::new(
            event
                .into_par_iter()
                .enumerate()
                .flat_map(|(round_num, round)| Some((round_num, round.event?)))
                .map(|(round_num, event)| {
                    let holes = holes.get(round_num).expect("hole should exist");
                    event
                        .players
                        .into_iter()
                        .flat_map(|player| {
                            let id = player.id.clone().into_inner();
                            Player::from_query(
                                player,
                                round_num,
                                holes,
                                divisions.clone(),
                                groups[round]
                                    .iter()
                                    .find(|group| {
                                        group
                                            .players
                                            .iter()
                                            .map(|player| player.id.as_str())
                                            .contains(&id.as_str())
                                    })
                                    .map(|group| group.start_at)
                                    .unwrap_or(0),
                            )
                        })
                        .collect()
                })
                .collect(),
            round,
        );

        for (i, round) in container.rounds_with_players.iter_mut().enumerate() {
            round.par_iter_mut().for_each(|player| {
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
            round_ind: round,
        })
    }

    pub fn get_previous_leaderboards(&self) -> Leaderboard {
        let mut lb = Leaderboard::default();

        if self.round_ind == 0 {
            return lb;
        }

        let previous_players = self.get_previous_rounds_players();

        for round in 0..self.round_ind {
            let state = LeaderboardState::new(
                round,
                previous_players
                    .clone()
                    .into_iter()
                    .filter(|player| player.round_ind == round)
                    .cloned()
                    .collect_vec(),
                previous_players
                    .clone()
                    .into_iter()
                    .filter(|player| player.round_ind == round.checked_sub(1).unwrap_or(1000))
                    .cloned()
                    .collect_vec(),
            );
            lb.add_state(state)
        }
        lb
    }

    pub fn add_total_score_to_players(&mut self) {
        let mut players = self
            .get_previous_rounds_players()
            .into_iter()
            .cloned()
            .collect_vec();
        for player in players.iter_mut() {
            player.fix_round_score(None);
        }
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
                return Err(Error::UnableToParse);
            };
            let Some(event) = data.event else {
                return Err(Error::UnableToParse);
            };
            let mut rounds_holes = vec![];
            for round in event.rounds {
                let Some(round) = round else {
                    return Err(Error::UnableToParse);
                };
                let holes = round
                    .pools
                    .into_iter()
                    .flat_map(|pool| pool.layout_version.holes)
                    .collect_vec();
                match Holes::from_vec_hole(holes) {
                    Err(e) => return Err(e),
                    Ok(holes) => rounds_holes.push(holes),
                }
            }
            rounds_holes
        };

        Ok(holes)
    }

    pub fn groups(&self) -> &Vec<dto::Group> {
        self.groups.get(self.round_ind).unwrap()
    }

    pub fn get_divisions(&self) -> Vec<Arc<queries::Division>> {
        self.divisions.clone()
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

    pub fn find_player_mut(&mut self, player_id: &str) -> Option<&mut Player> {
        self.player_container
            .players_mut()
            .into_iter()
            .find(|player| player.player_id == player_id)
    }

    pub fn get_players(&self) -> Vec<&Player> {
        self.player_container.players().iter().collect_vec()
    }

    pub fn get_previous_rounds_players(&self) -> Vec<&Player> {
        self.player_container.previous_rounds_players()
    }

    pub fn get_players_mut(&mut self) -> Vec<&mut Player> {
        self.player_container.players_mut()
    }
}
