mod vmix_calls;
mod simple_queries;

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
    pub score_card: ScoreCard,
    selected_div_ind: usize,
    pub selected_div: cynic::Id,
    leaderboard: Leaderboard,
    foc_play_ind: usize,
    ip: String,
    event_id: String,
    pools: Vec<queries::Pool>,
    handler: Option<RustHandler>,
    available_players: Vec<Player>,
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
            foc_play_ind: 0,
            ip: "".to_string(),
            pools: vec![],
            event_id: "75cceb0e-5a1d-4fba-a5c8-f2ff95f84495".into(),
            handler: None,
            available_players: vec![],
            round_ind: 0,
            lb_div_ind: 0,
            lb_thru: 0,
            score_card: ScoreCard::default(),
            queue,
            leaderboard: Leaderboard::default(),
        }
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

    fn find_same(&self, player: &Player) -> Option<Player> {
        for p in &self.score_card.all_play_players {
            if p.player_id == player.player_id {
                return Some({
                    let mut cl = p.clone();
                    cl.ind = player.ind;
                    cl
                });
            }
        }
        None
    }

    fn queue_add<T: VMixSelectionTrait>(&self, funcs: &[VMixFunction<T>]) {
        println!(
            "hello i am adding: {:#?}",
            funcs.iter().map(|f| f.to_cmd()).collect_vec()
        );
        self.queue.add(funcs)
    }

    fn set_lb_thru(&mut self) {
        let focused_players = [
            &self.score_card.p1,
            &self.score_card.p2,
            &self.score_card.p3,
            &self.score_card.p4,
        ];
        self.lb_thru = focused_players.iter().map(|p| p.hole).min().unwrap_or(0);
    }

    fn make_checkin_text(&self) -> VMixFunction<LeaderBoardProperty> {
        let value = self.get_div_names()[self.selected_div_ind].to_string()
            .to_uppercase()
            + " "
            + "LEADERBOARD CHECK-IN";
        VMixFunction::SetText {
            value,
            input: LeaderBoardProperty::CheckinText.into(),
        }
    }

    fn set_hot_round(&mut self) {
        let hot_round = self
            .score_card
            .all_play_players
            .iter()
            .map(|player| player.round_score)
            .min()
            .unwrap_or(0);
        for player in &mut self.score_card.all_play_players {
            if player.round_score == hot_round {
                player.hot_round = true;
            }
        }
    }

    fn make_lb(&mut self) -> Vec<VMixFunction<LeaderBoardProperty>> {
        let mut r_vec: Vec<VMixFunction<LeaderBoardProperty>> = self
            .score_card
            .all_play_players
            .iter_mut()
            .flat_map(|player| player.set_lb())
            .collect();
        r_vec.push(self.make_checkin_text());
        r_vec
    }

    pub fn toggle_pos(&mut self) {
        let f = self.get_focused_mut().toggle_pos();
        self.queue_add(&f)
    }

    fn get_focused_mut(&mut self) -> &mut get_data::Player {
        match self.foc_play_ind {
            0 => &mut self.score_card.p1,
            1 => &mut self.score_card.p2,
            2 => &mut self.score_card.p3,
            3 => &mut self.score_card.p4,
            _ => &mut self.score_card.p1,
        }
    }
    
    fn get_focused(&self) -> &Player {
        match self.foc_play_ind {
            0 => &self.score_card.p1,
            1 => &self.score_card.p2,
            2 => &self.score_card.p3,
            3 => &self.score_card.p4,
            _ => &self.score_card.p1
        }
    }
    
}

// API Funcs
// basically leftover from WASM

impl FlipUpVMixCoordinator {
    pub fn set_ip(&mut self, ip: String) {
        self.ip.clone_from(&ip);
        self.score_card.ip = ip;
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

    pub fn set_leaderboard(&mut self, update_players: bool, lb_start_ind: Option<usize>) {
        let mut return_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        self.queue_add(&FlipUpVMixCoordinator::clear_lb(10));
        println!("set_leaderboard");
        //let mut lb_copy = self.clone();
        self.set_lb_thru();
        println!("past set_lb_thru");
        if self.current_hole() <= 19 {
            println!("hole <= 19");
            self.lb_div_ind = self.selected_div_ind;
            self.fetch_players(true);
            println!(
                "{:#?}",
                self.score_card
                    .all_play_players
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<String>>()
            );
            println!("past get_players");

            if let Some(pop) = lb_start_ind {
                self.score_card.all_play_players.drain(0..pop);
                for player in &mut self.score_card.all_play_players {
                    player.position -= pop;
                }
            }

            self.leaderboard.update_players(LeaderboardState::new(
                self.round_ind,
                self.score_card.all_play_players.clone(),
            ));
            self.queue_add(&self.leaderboard.to_vmix_instructions());

            let players = [
                &self.score_card.p1,
                &self.score_card.p2,
                &self.score_card.p3,
                &self.score_card.p4,
            ];
            for player in players {
                let same = self.find_same(player);
                let mut cloned_player = player.clone();
                if cloned_player.hole > 7 {
                    cloned_player.hole -= 1;
                    let shift_scores = cloned_player.shift_scores(true);
                    if update_players {
                        return_vec.extend(shift_scores)
                    };
                }
                if let Some(same) = same {
                    if let Some(pos) = same.set_pos() {
                        return_vec.push(pos);
                    }
                }
            }
            // return_vec.push(self.find_same(&self.score_card.p1).unwrap().set_pos());
            // return_vec.push(self.find_same(&self.score_card.p2).unwrap().set_pos());
            // return_vec.push(self.find_same(&self.score_card.p3).unwrap().set_pos());
            // return_vec.push(self.find_same(&self.score_card.p4).unwrap().set_pos());
        } else {
            println!("PANIC, hole > 18");
        }
        self.queue_add(&return_vec);
    }

    pub fn set_all_to_hole(&mut self, hole: usize) {
        let instructions = [
            &mut self.score_card.p1,
            &mut self.score_card.p2,
            &mut self.score_card.p3,
            &mut self.score_card.p4,
        ]
        .iter_mut()
        .flat_map(|player| {
            if hole >= 9 {
                player.hole = hole - 1;
                player.shift_scores(true)
            } else {
                let mut r_vec: Vec<VMixFunction<VMixProperty>> = vec![];
                for x in 1..=hole {
                    println!("hello im here: {}", x);
                    player.hole = x;
                    r_vec.extend(player.set_hole_score());
                }
                r_vec
            }
        })
        .collect::<Vec<_>>();
        self.queue_add(&instructions)
    }




    pub fn set_round(&mut self, idx: usize) {
        self.round_ind = idx;
        self.lb_thru = 0;
        let funcs = self.score_card.set_round(idx);
        self.queue_add(&funcs);
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
    }

    pub fn increase_score(&mut self) {
        let hole = self.get_focused_mut().hole;
        self.hide_pos();
        //println!(format!("hole: {}", hole).as_str());
        if hole <= 17 {
            let f = {
                let focused = self.get_focused_mut();
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
        let f = self.get_focused_mut().hide_pos();
        self.queue_add(&f)
    }

    pub fn hide_all_pos(&mut self) {
        let out = self
            .score_card
            .get_all_player_mut()
            .map(Player::hide_pos)
            .into_iter()
            .flatten()
            .collect_vec();
        self.queue_add(&out);
    }

    

    pub fn ob_anim(&mut self) {
        println!("ob_anim");
        self.get_focused_mut().ob = true;
        if let Some(score) = self.get_focused_mut().get_score() {
            self.queue_add(&score.play_mov_vmix(self.foc_play_ind, true))
        }
    }
    pub fn set_player(&mut self, idx: usize, player: &str) {
        self.score_card.set_player(idx, player, self.round_ind);
    }

    pub fn set_foc(&mut self, idx: usize) {
        self.foc_play_ind = idx;
    }
    pub fn revert_score(&mut self) {
        let f = self.get_focused_mut().revert_hole_score();
        self.queue_add(&f);
    }
    pub fn reset_score(&mut self) {
        self.lb_thru = 0;
        let f = self.get_focused_mut().reset_scores();
        self.queue_add(&f)
    }

    pub fn reset_scores(&mut self) {
        let mut return_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        return_vec.extend(self.score_card.p1.reset_scores());
        return_vec.extend(self.score_card.p2.reset_scores());
        return_vec.extend(self.score_card.p3.reset_scores());
        return_vec.extend(self.score_card.p4.reset_scores());
        self.queue_add(&FlipUpVMixCoordinator::clear_lb(10));
        self.queue_add(&return_vec);
    }

    pub fn get_foc_p_name(&mut self) -> String {
        self.get_focused_mut().name.clone()
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
        if !lb {
            self.score_card.all_play_players = self.available_players.clone();
        }
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
            .map(|player| player.player_id.inner().to_owned())
            .collect()
    }

    pub fn increase_throw(&mut self) {
        self.get_focused_mut().throws += 1;
        let f = [self.get_focused_mut().set_throw()];
        self.queue_add(&f)
    }

    pub fn decrease_throw(&mut self) {
        self.get_focused_mut().throws -= 1;
        let f = &[self.get_focused_mut().set_throw()];
        self.queue_add(f);
    }

    pub fn get_focused_player_names(&self) -> Vec<&str> {
        [
            &self.score_card.p1,
            &self.score_card.p2,
            &self.score_card.p3,
            &self.score_card.p4,
        ]
        .iter()
        .map(|player| player.name.as_ref())
        .collect()
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
                    new.set_player(i, &player);
                });
            new.set_foc(0);
            new.set_round(self.round_ind);
            if self.lb_thru > 0 {
                new.set_all_to_hole(self.lb_thru - 1);
            } else {
                new.set_all_to_hole(0);
            }
            new.set_leaderboard(false, None);
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ScoreCard {
    pub players: [get_data::Player; 4],
    pub p1: get_data::Player,
    pub p2: get_data::Player,
    pub p3: get_data::Player,
    pub p4: get_data::Player,
    pub all_play_players: Vec<get_data::Player>,
    ip: String,
    queue: Option<Arc<Queue>>,
}

// Public scorecard funcs

impl ScoreCard {
    pub fn set_player(&mut self, player_num: usize, player_id: &str, rnd: usize) {
        //let player_id = player_id.trim_start_matches("\"").trim_end_matches("\"").to_string();
        let mut out_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        for player in self.all_play_players.clone() {
            if player.player_id == cynic::Id::from(player_id) {
                let mut p = player.clone();
                p.ind = player_num - 1;
                out_vec.extend(p.clone().set_name());
                out_vec.extend(p.clone().set_round(rnd)); // resets score and sets round
                match player_num {
                    1 => self.p1 = p.clone(),
                    2 => self.p2 = p.clone(),
                    3 => self.p3 = p.clone(),
                    4 => self.p4 = p.clone(),
                    _ => (),
                }
            }
        }
        println!("player_id: {}", player_id);

        self.queue_add(&out_vec).expect("Queue should exist");
    }

    fn queue_add<T: VMixSelectionTrait>(&self, functions: &[VMixFunction<T>]) -> Result<(), ()> {
        if let Some(q) = &self.queue {
            q.add(functions);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn set_total_score(&mut self, player_num: usize, new_score: isize) {
        match player_num {
            1 => self.p1.total_score = new_score,
            2 => self.p2.total_score = new_score,
            3 => self.p3.total_score = new_score,
            4 => self.p4.total_score = new_score,
            _ => panic!("Invalid player number"),
        }
    }
}

impl ScoreCard {
    fn set_round(&mut self, round: usize) -> Vec<VMixFunction<VMixProperty>> {
        let mut return_vec: Vec<VMixFunction<VMixProperty>> = vec![];

        return_vec.extend(self.p1.set_round(round));
        return_vec.extend(self.p2.set_round(round));
        return_vec.extend(self.p3.set_round(round));
        return_vec.extend(self.p4.set_round(round));
        return_vec
    }

    fn get_all_players_ref(&self) -> [&Player; 4] {
        [&self.p1, &self.p2, &self.p3, &self.p4]
    }

    fn get_all_player_mut(&mut self) -> [&mut Player; 4] {
        [&mut self.p1, &mut self.p2, &mut self.p3, &mut self.p4]
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
