mod simple_queries;
mod vmix_calls;

pub use super::*;
use crate::controller::get_data::RustHandler;
use crate::flipup_vmix_controls;
use crate::vmix;
use flipup_vmix_controls::LeaderBoardProperty;
use flipup_vmix_controls::{Leaderboard, LeaderboardState};
use get_data::{HoleScoreOrDefault, Player};
use itertools::Itertools;
use log::warn;
use std::sync::Arc;
use vmix::functions::VMixFunction;
use vmix::functions::{VMixProperty, VMixSelectionTrait};
use vmix::Queue;

mod old_public {
    pub fn greet() {
        println!("Hello, wasm-test!");
    }
}

#[derive(Clone, Debug)]
pub struct FlipUpVMixCoordinator {
    pub all_divs: Vec<queries::PoolLeaderboardDivision>,
    selected_div_ind: usize,
    pub selected_div: cynic::Id,
    leaderboard: Leaderboard,
    focused_player_index: usize,
    ip: String,
    event_id: String,
    pools: Vec<queries::Pool>,
    handler: Option<RustHandler>,
    pub available_players: Vec<Player>,
    round_ind: usize,
    lb_div_ind: usize,
    lb_thru: usize,
    pub queue: Arc<Queue>,
}
impl Default for FlipUpVMixCoordinator {
    fn default() -> FlipUpVMixCoordinator {
        let queue = Queue::new("10.170.120.134".to_string()).into(); // This is your main async runtime
        FlipUpVMixCoordinator {
            all_divs: vec![],
            selected_div_ind: 0,
            selected_div: cynic::Id::from("cd094fcf-76c7-471e-bfe3-be3d5892bd81"),
            focused_player_index: 0,
            ip: "".to_string(),
            pools: vec![],
            event_id: "75cceb0e-5a1d-4fba-a5c8-f2ff95f84495".into(),
            handler: None,
            available_players: vec![],
            round_ind: 0,
            lb_div_ind: 0,
            lb_thru: 0,
            queue,
            leaderboard: Leaderboard::default(),
        }
    }
}

impl FlipUpVMixCoordinator {
    pub fn new(
        ip: String,
        event_id: String,
        selected_division: String,
        focused_player: usize,
    ) -> Self {
        let queue = Queue::new(ip.clone());
        FlipUpVMixCoordinator {
            all_divs: vec![],
            selected_div_ind: 0,
            selected_div: cynic::Id::from(selected_division),
            focused_player_index: focused_player,
            ip,
            event_id,
            pools: vec![],
            handler: None,
            available_players: vec![],
            round_ind: 0,
            lb_div_ind: 0,
            lb_thru: 0,
            queue: Arc::new(queue),
            leaderboard: Leaderboard::default(),
        }
    }

    pub fn focused_player(&self) -> &Player {
        self.available_players
            .get(self.focused_player_index)
            .unwrap()
    }

    pub fn focused_player_mut(&mut self) -> &mut Player {
        self.available_players
            .get_mut(self.focused_player_index)
            .unwrap()
    }
}

impl FlipUpVMixCoordinator {
    // Initialise main app
    fn clear_lb(idx: usize) -> Vec<VMixFunction<LeaderBoardProperty>> {
        let mut new_player = get_data::Player::default();
        new_player.lb_vmix_id = "2ef7178b-61ab-445c-9bbd-2f1c2c781e86".into();

        let mut r_v: Vec<VMixFunction<LeaderBoardProperty>> = vec![];
        for i in 0..=idx {
            new_player.position = i;
            r_v.extend(new_player.set_lb());
        }
        r_v
    }

    fn queue_add<T: VMixSelectionTrait>(&self, funcs: &[VMixFunction<T>]) {
        println!(
            "hello i am adding: {:#?}",
            funcs.iter().map(|f| f.to_cmd()).collect_vec()
        );
        self.queue.add(funcs)
    }

    fn set_lb_thru(&mut self) {
        self.lb_thru = self.focused_player().thru
    }

    fn make_checkin_text(&self) -> VMixFunction<LeaderBoardProperty> {
        let value = self.get_div_names()[self.selected_div_ind]
            .to_string()
            .to_uppercase()
            + " "
            + "LEADERBOARD CHECK-IN";
        VMixFunction::SetText {
            value,
            input: LeaderBoardProperty::CheckinText.into(),
        }
    }

    pub fn toggle_pos(&mut self) {
        let f = self.focused_player_mut().toggle_pos();
        self.queue_add(&f)
    }
}

// API Funcs
// basically leftover from WASM

impl FlipUpVMixCoordinator {
    pub fn set_ip(&mut self, ip: String) {
        self.ip.clone_from(&ip);
        println!("ip set to {}", &self.ip);
    }
    pub fn set_div(&mut self, idx: usize) {
        println!("div set to {}", idx);
        self.selected_div_ind = idx;
        if let Some(handler) = &mut self.handler {
            handler.set_chosen_by_ind(idx);
        }
        self.fetch_players(false);
    }

    pub fn set_leaderboard(&mut self, lb_start_ind: Option<usize>) {
        self.queue_add(&FlipUpVMixCoordinator::clear_lb(10));
        println!("set_leaderboard");
        //let mut lb_copy = self.clone();
        self.set_lb_thru();
        println!("past set_lb_thru");
        if self.current_hole() <= 19 {
            println!("hole <= 19");
            self.lb_div_ind = self.selected_div_ind;
            self.fetch_players(true);
            if let Some(pop) = lb_start_ind {
                self.available_players.drain(0..pop);
                for player in &mut self.available_players {
                    player.position -= pop;
                }
            }

            self.leaderboard.update_players(LeaderboardState::new(
                self.round_ind,
                self.available_players.clone(),
            ));
            self.queue_add(&self.leaderboard.to_vmix_instructions());
        } else {
            println!("PANIC, hole > 18");
        }
    }

    pub fn set_to_hole(&mut self, hole: usize) {
        let player = self.focused_player_mut();
        let mut actions = vec![];
        if hole >= 9 {
            player.hole = hole - 1;
            actions.extend(player.shift_scores(true));
        } else {
            for x in 1..=hole {
                player.hole = x;
                actions.extend(player.set_hole_score());
            }
        }
        self.queue_add(&actions)
    }

    pub fn set_round(&mut self, idx: usize) {
        self.round_ind = idx;
        self.lb_thru = 0;
        let actions = self.focused_player_mut().set_round(idx);
        self.queue_add(&actions);
    }

    pub async fn fetch_event(&mut self) {
        self.pools = vec![];
        self.handler = Some(RustHandler::new(
            get_data::post_status(cynic::Id::from(&self.event_id)).await,
        ));

        match self.handler.clone() {
            Some(..) => println!("handler fine"),
            None => println!("handler on fire"),
        }
        self.available_players
            .extend(self.handler.clone().unwrap().get_players())
    }

    pub fn increase_score(&mut self) {
        let hole = self.focused_player_mut().hole;
        self.hide_pos();
        //println!(format!("hole: {}", hole).as_str());
        if hole <= 17 {
            let f = {
                let focused = self.focused_player_mut();
                if hole <= 7 {
                    focused.set_hole_score()
                } else {
                    focused.shift_scores(false)
                }
            };
            self.queue_add(&f);
        }
    }

    pub fn hide_pos(&mut self) {
        let f = self.focused_player_mut().hide_pos();
        self.queue_add(&f)
    }

    pub fn hide_all_pos(&mut self) {
        let out = self.focused_player_mut().hide_pos();
        self.queue_add(&out);
    }

    pub fn ob_anim(&mut self) {
        println!("ob_anim");
        self.focused_player_mut().ob = true;
        if let Some(score) = self.focused_player_mut().get_score() {
            self.queue_add(&score.play_mov_vmix(self.focused_player_index, true))
        }
    }
    pub fn set_player(&mut self, player: &str) {
        let index = self
            .available_players
            .iter()
            .enumerate()
            .find(|(_, p)| p.player_id == player)
            .map(|(i, _)| i)
            .unwrap();

        self.focused_player_index = index;
        let round = self.round_ind;
        let player = self.focused_player_mut();

        player.ind = 0;
        let mut actions = vec![];
        actions.extend(player.set_name());
        actions.extend(player.set_round(round));
        self.queue_add(&actions)
    }

    pub fn set_foc(&mut self, idx: usize) {
        self.focused_player_index = idx;
    }
    pub fn revert_score(&mut self) {
        let f = self.focused_player_mut().revert_hole_score();
        self.queue_add(&f);
    }
    pub fn reset_score(&mut self) {
        self.lb_thru = 0;
        let f = self.focused_player_mut().reset_scores();
        self.queue_add(&f)
    }

    pub fn reset_scores(&mut self) {
        let return_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        let actions = self.focused_player_mut().reset_scores();
        self.queue_add(&actions);
        self.queue_add(&FlipUpVMixCoordinator::clear_lb(10));
        self.queue_add(&return_vec);
    }

    pub fn get_div_names(&self) -> Vec<String> {
        let mut return_vec = vec![];

        for div in self.handler.clone().expect("handler!").get_divisions() {
            return_vec.push(div.name.clone());
        }
        return_vec
    }

    pub fn fetch_players(&mut self, lb: bool) {
        self.available_players = self.handler.clone().expect("handler!").get_players();
    }
    pub fn get_player_names(&self) -> Vec<String> {
        self.available_players
            .iter()
            .map(|player| player.name.clone())
            .collect()
    }

    pub fn get_player_ids(&self) -> Vec<String> {
        self.available_players
            .iter()
            .map(|player| player.player_id.to_owned())
            .collect()
    }

    pub fn increase_throw(&mut self) {
        self.focused_player_mut().throws += 1;
        let f = [self.focused_player_mut().set_throw()];
        self.queue_add(&f)
    }

    pub fn decrease_throw(&mut self) {
        self.focused_player_mut().throws -= 1;
        let f = &[self.focused_player_mut().set_throw()];
        self.queue_add(f);
    }

    pub fn get_focused_player_name(&self) -> &str {
        &self.focused_player().name
    }

    /// TODO: Refactor out into api function
    pub fn set_event_id(&mut self, event_id: String) {
        self.event_id = String::from(event_id);
    }

    pub fn make_separate_lb(&mut self, div_ind: usize) {
        if self.lb_thru != 0 {
            let mut new = self.clone();
            new.set_div(div_ind);
            new.fetch_players(false);
            new.available_players
                .iter_mut()
                .for_each(|player| player.visible_player = false);
            let players = new.get_player_ids();

            players
                .into_iter()
                .enumerate()
                .take(4 + 1)
                .skip(1)
                .for_each(|(i, player)| {
                    new.set_player(&player);
                });
            new.set_foc(0);
            new.set_round(self.round_ind);
            if self.lb_thru > 0 {
                new.set_to_hole(self.lb_thru - 1);
            } else {
                new.set_to_hole(0);
            }
            new.set_leaderboard(None);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests need to run with high node version otherwise it fails!

    use super::*;

    async fn generate_app() -> FlipUpVMixCoordinator {
        let mut app = FlipUpVMixCoordinator {
            event_id: "5c243af9-ea9d-4f44-ab07-9c55be23bd8c".to_string(),
            ..Default::default()
        };
        app.fetch_event().await.unwrap();
        println!("{:#?}", app.pools);
        app.set_div(0);
        app.fetch_players(false);
        let players = app.get_player_ids();
        // app.set_player(1, players[0].clone());
        // app.set_player(2, players[1].clone());
        // app.set_player(3, players[2].clone());
        // app.set_player(4, players[3].clone());
        // app.set_foc(1);
        players
            .iter()
            .enumerate()
            .take(4 + 1)
            .skip(1)
            .for_each(|(i, player)| {
                app.set_player(i, player.clone());
                //send(&handle_js_vec(test));
            });
        app.set_foc(1);
        app
    }
}
