use crate::api::websocket::HoleFinishedAlert;
use crate::api::GeneralChannel;
use crate::controller;
use crate::controller::coordinator::leaderboard_cycle::LeaderboardCycle;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::queries::results_getter::PlayerResults;
use crate::controller::queries::HoleResult;
use cynic::{GraphQlResponse, QueryBuilder};
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug)]
struct TjingResultMap {
    results: HashMap<
        String,
        Vec<Option<crate::controller::coordinator::queries::results_getter::HoleResult>>,
    >,
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
        hole_result: crate::controller::queries::results_getter::HoleResult,
    ) -> bool {
        let mut needs_update = false;
        let results = self.results.entry(player_id.to_string()).or_default();

        if let Some(res) = results
            .iter_mut()
            .flatten()
            .find(|hole| hole.hole_number as u8 == hole_result.hole_number as u8)
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
        hole_results: Vec<crate::controller::queries::results_getter::HoleResult>,
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
    pub fn update_all_players(&mut self, players: PlayerResults) -> bool {
        let mut needs_update = false;
        for (player_id, hole_results) in players.0.into_iter() {
            if self.update_many(&player_id.into_inner(), hole_results) {
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
    hole_finished_alert: GeneralChannel<HoleFinishedAlert>,
    next_group: Arc<Mutex<String>>,
) {
    let temp_coordinator = coordinator.lock().await;
    let mut tjing_result_map = TjingResultMap::new(temp_coordinator.available_players());

    let round_ids = temp_coordinator.round_ids();
    drop(temp_coordinator);

    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        for round_id in &round_ids {
            interval.tick().await;
            let Some(results) =
                controller::queries::results_getter::get_round_results(round_id.into()).await
            else {
                warn!("Failed to get results for round {}", round_id);
                continue;
            };
            if tjing_result_map.update_all_players(results) {
                let mut coordinator = coordinator.lock().await;
                coordinator
                    .available_players_mut()
                    .into_iter()
                    .for_each(|player| tjing_result_map.update_mut_player(player));
                let div = coordinator.focused_player().division.clone();
                let queue = coordinator.vmix_queue.clone();
                coordinator.add_state_to_leaderboard();
                //coordinator.leaderboard.update_little_lb(&div, queue);
                if let Some(player) =
                    coordinator
                        .available_players()
                        .into_par_iter()
                        .find_any(|player| {
                            player
                                .results
                                .latest_hole_finished()
                                .is_some_and(|hole| hole.hole == 18)
                        })
                {
                    if let Some((group_id, Some(division))) = coordinator
                        .groups()
                        .into_par_iter()
                        .find_any(|group| {
                            group
                                .players
                                .iter()
                                .any(|group_player| group_player.id == player.player_id)
                        })
                        .map(|group| {
                            (
                                group.id.clone(),
                                group.players.first().map(|player| player.division.clone()),
                            )
                        })
                    {
                        *next_group.lock().await = group_id.clone();
                        let cycle_mutex = leaderboard_cycle.clone();
                        cycle_mutex
                            .lock()
                            .await
                            .set_featured_div(division, group_id)
                            .await;
                    }
                    hole_finished_alert.send(HoleFinishedAlert::JustFinished);

                    let alert = hole_finished_alert.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(2 * 60)).await;
                        alert.send(HoleFinishedAlert::SecondSend)
                    });
                }
                drop(coordinator);
                leaderboard_cycle
                    .clone()
                    .lock()
                    .await
                    .update_leaderboard()
                    .await;
            }

            tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;
        }
    }
}
