use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::queries::Division;
use crate::flipup_vmix_controls::Leaderboard;
use crate::vmix::VMixQueue;
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
}

impl LeaderboardCycle {
    async fn new(coordinator: Arc<Mutex<FlipUpVMixCoordinator>>) -> Self {
        let temp_coordinator = coordinator.lock().await;
        let all_divisions = VecDeque::from(temp_coordinator.handler.get_divisions().clone());
        let round = temp_coordinator.round_ind;
        let mut leaderboard = temp_coordinator.handler.get_previous_leaderboards();
        leaderboard.cycle = true;
        drop(temp_coordinator);
        Self {
            current_cycled: all_divisions.front().unwrap().clone(),
            all_divisions,
            coordinator,
            leaderboard,
            round,
        }
    }

    pub async fn update_leaderboard(&mut self) {
        let coordinator = self.coordinator.lock().await;
        let state = coordinator.current_leaderboard_state();
        self.leaderboard.add_state(state);
        let queue = coordinator.vmix_queue.clone();
        self.leaderboard
            .send_to_vmix(&self.current_cycled, queue, self.round)
    }

    fn send(&self, queue: Arc<VMixQueue>, round: usize) {
        self.leaderboard
            .send_to_vmix(&self.current_cycled, queue, round)
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
