use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::queries::Division;
use crate::flipup_vmix_controls::Leaderboard;
use crate::vmix::functions::VMixInterfacer;
use crate::vmix::VMixQueue;
use itertools::Itertools;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct LeaderboardCycle {
    current_cycled: Arc<Division>,
    all_divisions: VecDeque<Arc<Division>>,
    coordinator: Arc<Mutex<FlipUpVMixCoordinator>>,
    leaderboard: Leaderboard,
    round: usize,
    current_featured_div: Arc<Division>,
}

impl LeaderboardCycle {
    async fn new(coordinator: Arc<Mutex<FlipUpVMixCoordinator>>) -> Self {
        let temp_coordinator = coordinator.lock().await;
        let all_divisions = VecDeque::from(temp_coordinator.handler.get_divisions().clone());
        let round = temp_coordinator.round_ind;
        let mut leaderboard = temp_coordinator.handler.get_previous_leaderboards();
        leaderboard.cycle = true;
        let first_featured_div = temp_coordinator
            .featured_card
            .players(temp_coordinator.available_players())
            .first()
            .unwrap()
            .division
            .clone();
        drop(temp_coordinator);
        Self {
            current_cycled: all_divisions.front().unwrap().clone(),
            all_divisions,
            coordinator,
            leaderboard,
            round,
            current_featured_div: first_featured_div,
        }
    }

    pub async fn set_featured_div(&mut self, division: Arc<Division>, group_id: String) {
        self.current_featured_div = division.clone();
        let mut coordinator = self.coordinator.lock().await;
        let state = coordinator.current_leaderboard_state();
        self.leaderboard.add_state(state);
        let queue = coordinator.vmix_queue.clone();
        let players = coordinator
            .groups()
            .iter()
            .find(|group| group.id == group_id)
            .map(|group| group.players.clone())
            .unwrap_or({
                warn!("USING DEFAULT PAR 3");
                vec![]
            });

        let stats = coordinator.make_stats();
        if let Some(player) = coordinator.find_player_mut(&players[0].id) {
            let holes = player.holes.clone();
            let out = player
                .results
                .get_hole_info(1, stats, &holes, &division)
                .into_iter()
                .map(VMixInterfacer::into_featured_hole_card)
                .collect_vec();

            queue.add(out.into_iter());
        }

        self.leaderboard
            .send_to_vmix(&self.current_featured_div, queue, self.round, true);
    }
    fn refresh_leaderboard(&mut self, queue: Arc<VMixQueue>) {
        self.leaderboard
            .send_to_vmix(&self.current_cycled, queue, self.round, false)
    }

    pub async fn update_leaderboard(&mut self) {
        let coordinator = self.coordinator.lock().await;
        let state = coordinator.current_leaderboard_state();
        self.leaderboard.add_state(state);
        let queue = coordinator.vmix_queue.clone();
        drop(coordinator);
        self.refresh_leaderboard(queue);
    }

    fn send(&self, queue: Arc<VMixQueue>, round: usize) {
        self.leaderboard
            .send_to_vmix(&self.current_cycled, queue, round, false)
    }

    pub async fn next(&mut self) {
        let coordinator = self.coordinator.lock().await;
        let queue = coordinator.vmix_queue.clone();
        self.leaderboard
            .add_state(coordinator.current_leaderboard_state());
        drop(coordinator);
        self.current_cycled = self.cycle_next(queue);
    }

    fn cycle_next(&mut self, queue: Arc<VMixQueue>) -> Arc<Division> {
        let div = self.all_divisions.pop_front().unwrap();
        if self.current_featured_div == div {
            return self.cycle_next(queue);
        }
        self.all_divisions.push_back(div.clone());
        let all_players = self.leaderboard.all_players_in_div(div.clone(), self.round);
        if all_players.is_empty() {
            self.cycle_next(queue)
        } else {
            if all_players.len() < 6 {
                queue.add(FlipUpVMixCoordinator::clear_little_cycling_lb().into_iter())
            }
            div
        }
    }
}

pub async fn start_leaderboard_cycle(
    coordinator: Arc<Mutex<FlipUpVMixCoordinator>>,
) -> Arc<Mutex<LeaderboardCycle>> {
    let cycle = Arc::new(Mutex::new(LeaderboardCycle::new(coordinator).await));
    let loop_cycle = cycle.clone();
    tokio::spawn(async move {
        let cycle = loop_cycle;
        loop {
            let mut cycle = cycle.lock().await;
            cycle.next().await;
            cycle.update_leaderboard().await;
            tokio::time::sleep(Duration::from_secs(20)).await;
        }
    });
    cycle
}
