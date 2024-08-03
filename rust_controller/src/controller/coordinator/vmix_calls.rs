use std::collections::HashMap;

use itertools::Itertools;

use crate::api::Error;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::hole::HoleStats;
use crate::controller::queries;

impl FlipUpVMixCoordinator {
    pub fn make_hole_info(&mut self) {
        self.set_current_through();
        if self.current_hole() <= 18 {
            let current_hole = (self.current_hole() + 1) as u8;
            let stats = self.make_stats();
            let holes = self.focused_player().holes.clone();
            let result =
                self.focused_player_mut()
                    .results
                    .get_hole_info(current_hole, stats, &holes);
            let drone_result = self
                .focused_player()
                .results
                .get_drone_info(self.current_hole() as u8, &result);
            self.queue_add(&result);
            self.queue_add(&drone_result);
        }
    }

    pub(crate) fn make_stats(&self) -> Vec<HoleStats> {
        let mut hole_stats: HashMap<usize, Vec<queries::HoleResult>> = HashMap::new();
        self.available_players().into_iter().for_each(|player| {
            for (hole, result) in player
                .results
                .to_owned()
                .tjing_results()
                .into_iter()
                .enumerate()
            {
                if let Some(result) = result {
                    hole_stats.entry(hole).or_default().push(result);
                }
            }
        });
        hole_stats
            .into_iter()
            .sorted_by_key(|(hole, _)| hole.to_owned())
            .map(|(hole, results)| HoleStats::new(hole as u8, results))
            .collect()
    }

    pub fn play_animation(&mut self) -> Result<(), Error> {
        let score = self.focused_player_mut().get_current_shown_score();
        self.queue_add(&score.play_mov_vmix(self.focused_player_index, false));
        Ok(())
    }
}
