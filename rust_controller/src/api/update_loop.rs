use crate::controller;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::queries::{Division, HoleResult, RoundResultsQuery, RoundResultsQueryVariables};
use crate::controller::Player;
use cynic::{GraphQlResponse, QueryBuilder};
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
struct TjingResultMap {
    results: HashMap<String, Vec<Option<crate::controller::coordinator::queries::HoleResult>>>,
}

impl TjingResultMap {
    pub fn new(players: Vec<&Player>) -> Self {
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
        if let Some(results) = self.results.get_mut(player_id) {
            for result in results {
                if let Some(result) = result {
                    if result.hole.number == hole_result.hole.number {
                        *result = hole_result;
                        needs_update = true;
                        break;
                    }
                } else {
                    *result = Some(hole_result);
                    needs_update = true;
                    break;
                }
            }
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
        let hash_player_result = self.results.get(&player.player_id).into_iter().flat_map(|list|list.into_iter()).flatten().cloned().collect_vec();
        player.results.update_tjing(&hash_player_result);
    }
}

pub async fn update_loop(coordinator: Arc<Mutex<FlipUpVMixCoordinator>>) {
    let temp_coordinator = coordinator.lock().await;
    let mut tjing_result_map = TjingResultMap::new(temp_coordinator.available_players());

    let tjing_request = crate::controller::coordinator::queries::RoundResultsQuery::build(
        RoundResultsQueryVariables {
            round_id: temp_coordinator.round_id().to_owned().into(),
            event_id: temp_coordinator.event_id.to_owned().into(),
        },
    );
    let reqwest_client = reqwest::Client::new();
    std::mem::drop(temp_coordinator);
    loop {
        if let Ok(response) = reqwest_client
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
                    for player in coordinator.available_players_mut() {
                        tjing_result_map.update_mut_player(player)
                    }
                    /*if let Some(div) = coordinator.find_division_by_name("Mixed Amateur 1") {
                        coordinator.set_leaderboard(&div, None);
                    }*/
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
