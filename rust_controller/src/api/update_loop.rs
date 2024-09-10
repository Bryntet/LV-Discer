use std::collections::HashMap;
use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use itertools::Itertools;
use rayon::prelude::*;
use tokio::sync::Mutex;

use crate::controller;
use crate::controller::coordinator::leaderboard_cycle::LeaderboardCycle;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::queries::{HoleResult, RoundResultsQuery, RoundResultsQueryVariables};

#[derive(Debug)]
struct TjingResultMap {
    results: HashMap<String, Vec<Option<crate::controller::coordinator::queries::HoleResult>>>,
}

impl TjingResultMap {
    pub fn new(players: Vec<&controller::Player>) -> Self {
        Self {
            results: players
                .into_iter()
                .cloned()
                .map(|player| (player.player_id, player.results.tjing_results()))
                .collect(),
        }
    }

    #[inline(always)]
    pub fn update(
        &mut self,
        player_id: &str,
        hole_result: crate::controller::coordinator::queries::HoleResult,
    ) -> bool {
        let mut needs_update = false;
        let results = self.results.entry(player_id.to_string()).or_default();

        if let Some(res) = results
            .iter_mut()
            .flatten()
            .find(|hole| hole.hole.number as u8 == hole_result.hole.number as u8)
        {
            if res.score != hole_result.score {
                res.score = hole_result.score;
                needs_update = true;
            }
        } else {
            results.push(Some(hole_result));
            needs_update = true
        }

        needs_update
    }

    #[inline(always)]
    pub fn update_many(
        &mut self,
        player_id: &str,
        hole_results: Vec<crate::controller::coordinator::queries::HoleResult>,
    ) -> bool {
        let mut needs_update = false;
        for hole_result in hole_results {
            if self.update(player_id, hole_result) {
                needs_update = true;
            }
        }
        needs_update
    }

    #[inline(always)]
    pub fn update_all_players(&mut self, players: Vec<(String, Vec<HoleResult>)>) -> bool {
        let mut needs_update = false;
        for player in players {
            if self.update_many(&player.0, player.1) {
                needs_update = true;
            }
        }
        needs_update
    }

    pub fn update_mut_player(&self, player: &mut controller::Player) {
        let hash_player_result = self
            .results
            .get(&player.player_id)
            .into_iter()
            .flat_map(|list| list.iter())
            .flatten()
            .cloned()
            .collect_vec();
        player
            .results
            .update_tjing(&hash_player_result, &player.holes);
    }
}

pub async fn update_loop(
    coordinator: Arc<Mutex<FlipUpVMixCoordinator>>,
    leaderboard_cycle: Arc<Mutex<LeaderboardCycle>>,
) {
    let temp_coordinator = coordinator.lock().await;
    let mut tjing_result_map = TjingResultMap::new(temp_coordinator.available_players());

    let mut requests = vec![];
    let round_ids = temp_coordinator.round_ids();
    for (event_number, event_id) in temp_coordinator.event_ids.iter().enumerate() {
        let tjing_request = crate::controller::coordinator::queries::RoundResultsQuery::build(
            RoundResultsQueryVariables {
                round_id: round_ids[event_number].to_owned().into(),
                event_id: event_id.to_owned().into(),
            },
        );
        requests.push(tjing_request)
    }
    drop(temp_coordinator);

    loop {
        for tjing_request in &requests {
            if let Ok(response) = reqwest::Client::new()
                .post("https://api.tjing.se/graphql")
                .json(&tjing_request)
                .send()
                .await
            {
                if let Ok(GraphQlResponse {
                    data:
                        Some(RoundResultsQuery {
                            event: Some(controller::queries::Event { players, .. }),
                        }),
                    ..
                }) = response.json::<GraphQlResponse<RoundResultsQuery>>().await
                {
                    if tjing_result_map.update_all_players(
                        players
                            .into_iter()
                            .flat_map(|player| Some((player.id.into_inner(), player.results?)))
                            .collect_vec(),
                    ) {
                        let mut coordinator = coordinator.lock().await;
                        coordinator
                            .available_players_mut()
                            .into_iter()
                            .for_each(|player| tjing_result_map.update_mut_player(player));
                        let div = coordinator.focused_player().division.clone();
                        let queue = coordinator.vmix_queue.clone();
                        coordinator.add_state_to_leaderboard();
                        coordinator.leaderboard.update_little_lb(&div, queue);
                        drop(coordinator);
                        leaderboard_cycle.lock().await.update_leaderboard().await;
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;
        }
    }
}
