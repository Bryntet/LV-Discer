use cynic::GraphQlResponse;
use itertools::Itertools;
use log::warn;
use rayon::prelude::*;
use rocket::futures::{FutureExt, StreamExt};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use super::queries;
use crate::api::Error;
use crate::controller::coordinator::player::{Player, PlayerRound};
use crate::controller::coordinator::BroadcastType;
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
        let hole_representation = holes.find_hole(hole).ok_or(Error::UnableToParse)?;
        Ok(Self {
            hole,
            throws: hole_representation.par,
            hole_representation,
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
    ) -> Vec<VMixInterfacer<LeaderboardTop6>> {
        vec![
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

    pub fn to_current_player(
        &self,
        hole: usize,
        player: usize,
    ) -> Vec<VMixInterfacer<CurrentPlayer>> {
        vec![
            VMixInterfacer::set_text(
                fix_score(self.actual_score() as isize),
                CurrentPlayer(VMixPlayerInfo::Score { player, hole }),
            ),
            VMixInterfacer::set_color(
                self.to_score().get_score_colour(),
                CurrentPlayer(VMixPlayerInfo::ScoreColor { player, hole }),
            ),
        ]
    }

    pub fn hide_current_player_score(
        hole: usize,
        player: usize,
    ) -> Vec<VMixInterfacer<CurrentPlayer>> {
        vec![
            VMixInterfacer::set_text(
                "".to_string(),
                CurrentPlayer(VMixPlayerInfo::Score { hole, player }),
            ),
            VMixInterfacer::set_color(
                DEFAULT_FOREGROUND_COL_ALPHA,
                CurrentPlayer(VMixPlayerInfo::ScoreColor { hole, player }),
            ),
        ]
    }

    pub fn hide_hole_top_6(player: usize, hole: usize) -> Vec<VMixInterfacer<LeaderboardTop6>> {
        vec![
            VMixInterfacer::set_text(
                "".to_string(),
                LeaderboardTop6::LastScore { pos: player, hole },
            ),
            VMixInterfacer::set_color(
                DEFAULT_FOREGROUND_COL_ALPHA,
                LeaderboardTop6::LastScoreColour { hole, pos: player },
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
            player.vmix_index(),
            player.total_score,
        )
    }
}

#[derive(Clone, Debug)]
pub struct RustHandler {
    pub chosen_division: cynic::Id,
    pub round_ids: Vec<Vec<String>>,
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
    pub async fn new(
        event_ids: Vec<String>,
        round: usize,
        broadcast_type: Arc<BroadcastType>,
    ) -> Result<Self, Error> {
        let time = std::time::Instant::now();
        let round_ids = Self::get_rounds(&event_ids).await?;
        let events = Self::get_event(&event_ids, &round_ids).await;
        let groups = Self::get_groups(&event_ids).await;

        warn!("Time taken to get event: {:?}", time.elapsed());

        let conversion_names = [
            ("women's amateur 1", "FA1"),
            ("women's amateur 2", "FA2"),
            ("women's amateur 3", "FA3"),
            ("women's amateur 4", "FA4"),
            ("mixed amateur 1", "MA1"),
            ("mixed amateur 2", "MA2"),
            ("mixed amateur 3", "MA3"),
            ("mixed amateur 4", "MA4"),
            ("mixed amateur 40+", "MA40"),
            ("mixed amateur 50+", "MA50"),
        ];
        let division_name_conversion: HashMap<&'static str, &'static str> =
            HashMap::from(conversion_names);

        let sort_conversion: [(&'static str, usize); 10] = conversion_names
            .into_iter()
            .enumerate()
            .map(|(usize, (_, new_name))| (new_name, usize))
            .collect_vec()
            .try_into()
            .unwrap();
        let sort: HashMap<&str, usize> = HashMap::from(sort_conversion);
        let mut divisions: Vec<Arc<Division>> = events
            .iter()
            .flat_map(|event| {
                event
                    .iter()
                    .flat_map(|event| event.divisions.clone())
                    .flatten()
                    .map(|mut div| {
                        let div_name = div.name.to_lowercase();
                        div.name = division_name_conversion
                            .get(div_name.as_str())
                            .map(|name| name.to_string())
                            .unwrap_or(div.name);
                        div
                    })
            })
            .sorted_by_key(|div| *sort.get(div.name.as_str()).unwrap_or(&0))
            .dedup_by(|a, b| a.name == b.name)
            .map(Arc::new)
            .collect_vec();

        let holes = Self::get_holes(&event_ids).await?;

        let mut player_rounds: Vec<Vec<Player>> = vec![];
        for (event_number, event) in events.into_iter().enumerate() {
            let out = event
                .into_iter()
                .enumerate()
                .map(|(round_number, event)| {
                    event
                        .players
                        .into_iter()
                        // This validates that only players on the correct course are used
                        .filter_map(|player| {
                            let id = player.id.to_owned().into_inner();
                            groups[round_number]
                                .par_iter()
                                .find_any(|group| {
                                    group
                                        .players
                                        .iter()
                                        .map(|player| player.id.as_str())
                                        .contains(&id.as_str())
                                })
                                .map(|group| (player, group))
                        })
                        .map(|(mut player, group)| {
                            let div_name = player.division.name.to_lowercase();
                            player.division.name = division_name_conversion
                                .get(div_name.as_str())
                                .unwrap_or(&player.division.name.as_str())
                                .to_string();

                            (player, group)
                        })
                        .map(|(player, group)| {
                            let holes = match holes[event_number][round_number].get(&group.id) {
                                Some(holes) => holes.clone(),
                                None => {
                                    dbg!(player.division.name);
                                    panic!()
                                }
                            };

                            Player::from_query(
                                player,
                                round_number,
                                holes,
                                divisions.clone(),
                                group.start_at_hole,
                                event_number,
                                broadcast_type.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>()
                })
                .collect_vec();
            for (round_num, round_players) in out.into_iter().enumerate() {
                match player_rounds.get_mut(round_num) {
                    Some(players) => players.extend(round_players),
                    None => player_rounds.push(round_players),
                }
            }
        }

        let mut container = PlayerContainer::new(player_rounds, round);

        for (round_number, round) in container.rounds_with_players.iter_mut().enumerate() {
            round.par_iter_mut().for_each(|player| {
                if let Some(group_index) = groups[round_number].iter().find_map(|group| {
                    group
                        .players
                        .iter()
                        .filter(|player| !player.name.is_empty())
                        .enumerate()
                        .find_map(|(index, group_player)| {
                            if player.player_id == group_player.id {
                                Some(index)
                            } else {
                                None
                            }
                        })
                }) {
                    player.group_index = group_index;
                }
            });
        }

        Ok(Self {
            chosen_division: divisions.first().unwrap().id.clone(),
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
        event_ids: &[String],
        round_ids: &[Vec<String>],
    ) -> Vec<Vec<queries::Event>> {
        use cynic::QueryBuilder;
        use queries::*;
        let mut out = vec![];
        for (event_number, event_id) in event_ids.into_iter().enumerate() {
            let mut rounds = vec![];
            for round_id in &round_ids[event_number] {
                let operation = RoundResultsQuery::build(RoundResultsQueryVariables {
                    event_id: event_id.into(),
                    round_id: round_id.to_owned().into(),
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
                rounds.push(out.event.unwrap());
            }
            out.push(rounds);
        }
        out.try_into().unwrap()
    }

    pub async fn get_rounds(event_ids: &[String]) -> Result<Vec<Vec<String>>, Error> {
        use cynic::QueryBuilder;
        use queries::round::{RoundsQuery, RoundsQueryVariables};
        let mut out = vec![];
        for event_id in event_ids {
            let body = RoundsQuery::build(RoundsQueryVariables {
                event_id: event_id.to_string().into(),
            });

            let response = reqwest::Client::new()
                .post("https://api.tjing.se/graphql")
                .json(&body)
                .send()
                .await
                .map_err(|_| Error::UnableToParse)?
                .json::<GraphQlResponse<RoundsQuery>>()
                .await;
            out.push(
                response
                    .unwrap()
                    .data
                    .unwrap()
                    .event
                    .unwrap()
                    .rounds
                    .into_iter()
                    .flatten()
                    .map(|round| round.id.into_inner())
                    .collect_vec(),
            );
        }
        Ok(out.try_into().unwrap())
    }

    pub async fn get_holes(
        event_ids: &[String],
    ) -> Result<Vec<Vec<HashMap<String, Holes>>>, Error> {
        use cynic::QueryBuilder;
        use queries::layout::{HoleLayoutQuery, HoleLayoutQueryVariables};
        let mut out = vec![];
        for event_id in event_ids {
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

            let Some(Some(event)) = holes.data.map(|data| data.event) else {
                return Err(Error::UnableToParse);
            };

            let mut event_out = vec![];

            for round in event.rounds {
                let Some(round) = round else {
                    return Err(Error::UnableToParse);
                };
                let mut return_map: HashMap<String, Holes> = HashMap::new();
                for pool in round.pools {
                    let holes = Holes::from_vec_hole(pool.layout_version.holes)?;
                    for group in pool.groups {
                        return_map.insert(group.id.into_inner(), holes.clone());
                    }
                }
                event_out.push(return_map);
            }

            out.push(event_out);
        }
        Ok(out.try_into().unwrap())
    }

    pub fn groups(&self) -> &Vec<dto::Group> {
        self.groups.get(self.round_ind).unwrap()
    }

    pub fn get_divisions(&self) -> Vec<Arc<queries::Division>> {
        self.divisions.clone()
    }

    async fn get_groups(event_ids: &[String]) -> Vec<Vec<dto::Group>> {
        use cynic::QueryBuilder;
        use queries::group::{GroupsQuery, GroupsQueryVariables};
        let mut out: Vec<Vec<dto::Group>> = vec![];
        for event_id in event_ids {
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

            let group_rounds = groups
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
                        .sorted_by_key(|group| group.id.inner().to_owned())
                        .dedup_by(|group1, group2| group1.id == group2.id)
                        .map(dto::Group::from)
                        .collect_vec()
                })
                .collect::<Vec<Vec<dto::Group>>>();

            for (round_number, groups) in group_rounds.into_iter().enumerate() {
                while out.len() <= round_number {
                    out.push(vec![])
                }
                out[round_number].extend(groups);
            }
        }
        out
    }

    pub fn round_ids(&self) -> Vec<String> {
        self.round_ids
            .iter()
            .map(|events_rounds| events_rounds[self.round_ind].to_owned())
            .collect_vec()
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

    pub fn all_players(&self) -> Vec<&Player> {
        self.player_container
            .rounds_with_players
            .iter()
            .flatten()
            .collect_vec()
    }
}
