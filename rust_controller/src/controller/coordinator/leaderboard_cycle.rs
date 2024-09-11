use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::queries::Division;
use crate::flipup_vmix_controls::{Leaderboard, LeaderboardState};
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
}

impl LeaderboardCycle {
    async fn new(coordinator: Arc<Mutex<FlipUpVMixCoordinator>>) -> Self {
        let temp_coordinator = coordinator.lock().await;
        let all_divisions = VecDeque::from(temp_coordinator.all_divs.clone());

        let mut leaderboard = temp_coordinator.handler.get_previous_leaderboards();
        leaderboard.cycle = true;
        drop(temp_coordinator);
        Self {
            current_cycled: all_divisions.back().unwrap().clone(),
            all_divisions,
            coordinator,
            leaderboard,
        }
    }

    pub async fn update_leaderboard(&mut self) {
        let coordinator = self.coordinator.lock().await;
        let current_players = coordinator
            .available_players()
            .into_iter()
            .cloned()
            .collect_vec();
        let previous = coordinator
            .previous_rounds_players()
            .into_iter()
            .cloned()
            .collect_vec();
        self.leaderboard.add_state(LeaderboardState::new(
            coordinator.round_ind,
            current_players,
            previous,
        ));
        self.leaderboard
            .update_little_lb(&self.current_cycled, coordinator.vmix_queue.clone());
    }

    fn send(&self, queue: Arc<VMixQueue>, round: usize) {
        self.leaderboard
            .send_to_vmix(&self.current_cycled, queue, round)
    }

    pub async fn next(&mut self) {
        let coordinator = self.coordinator.lock().await;
        let current_division = coordinator.leaderboard_division.clone();
        drop(coordinator);
        let next = self.cycle_next();
        if next == current_division {
            self.current_cycled = self.cycle_next()
        } else {
            self.current_cycled = next;
        }
    }

    fn cycle_next(&mut self) -> Arc<Division> {
        let div = self.all_divisions.pop_front().unwrap();
        self.all_divisions.push_back(div.clone());
        div
    }
}

pub async fn start_leaderboard_cycle(
    coordinator: Arc<Mutex<FlipUpVMixCoordinator>>,
) -> Arc<Mutex<LeaderboardCycle>> {
    dbg!("here");
    let cycle = Arc::new(Mutex::new(LeaderboardCycle::new(coordinator).await));
    dbg!("after here");
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
