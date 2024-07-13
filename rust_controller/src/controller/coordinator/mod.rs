mod simple_queries;
mod vmix_calls;

pub use super::*;
use crate::api::{GeneralChannel, HoleUpdate, Error};
use crate::controller::get_data::RustHandler;
use crate::controller::queries::Division;
use crate::{api, vmix};
use crate::{dto, flipup_vmix_controls};
use flipup_vmix_controls::LeaderBoardProperty;
use flipup_vmix_controls::{Leaderboard, LeaderboardState};
use get_data::Player;
use itertools::Itertools;
use log::warn;
use rocket::http::hyper::body::HttpBody;
use rocket::tokio::sync::broadcast::Sender;
use rocket::State;
use std::ops::Deref;
use std::sync::Arc;
use vmix::functions::VMixFunction;
use vmix::functions::{VMixProperty, VMixSelectionTrait};
use vmix::Queue;

#[derive(Clone, Debug)]
pub struct FlipUpVMixCoordinator {
    pub all_divs: Vec<queries::Division>,
    selected_div_index: usize,
    leaderboard: Leaderboard,
    focused_player_index: usize,
    ip: String,
    handler: RustHandler,
    pub available_players: Vec<Player<'a>>,
    round_ind: usize,
    lb_div_ind: usize,
    current_through: usize,
    pub queue: Arc<Queue>,
    card: Card,
}
#[derive(Clone, Debug, Default)]
struct Card {
    player_ids: Vec<String>,
}
impl Card {
    fn new(player_ids: Vec<String>) -> Self {
        Self { player_ids }
    }

    fn player<'a>(&self, all_players: &'a [Player], index: usize) -> Option<&'a Player> {
        let player_id = self.player_ids.get(index)?;
        all_players
            .iter()
            .find(|player| &player.player_id == player_id)
    }

    fn players<'a>(&self, all_players: &'a [Player]) -> Vec<&'a Player> {
        self.player_ids
            .iter()
            .filter_map(|id| all_players.iter().find(|player| &player.player_id == id))
            .collect()
    }

    fn player_mut<'a>(
        &self,
        all_players: &'a mut [Player],
        index: usize,
    ) -> Option<&'a mut Player> {
        let player_id = self.player_ids.get(index)?;
        all_players
            .iter_mut()
            .find(|player| &player.player_id == player_id)
    }
}
impl FlipUpVMixCoordinator {
    pub async fn new(ip: String, event_id: String, focused_player: usize) -> Result<Self, Error> {
        let queue = Queue::new(ip.clone())?;
        let handler = RustHandler::new(&event_id).await?;

        let id = handler.groups.first().unwrap().first().unwrap().id.clone();
        let available_players = handler.clone().get_players();
        let mut coordinator = FlipUpVMixCoordinator {
            all_divs: vec![],
            selected_div_index: 0,
            focused_player_index: focused_player,
            ip,
            card: Card::new(
                handler
                    .groups
                    .first()
                    .unwrap()
                    .first()
                    .unwrap()
                    .player_ids(),
            ),
            handler,
            available_players,
            round_ind: 0,
            lb_div_ind: 0,
            current_through: 0,
            queue: Arc::new(queue),
            leaderboard: Leaderboard::default(),
        };
        coordinator.queue_add(&coordinator.focused_player().set_name());
        coordinator.reset_score();
        coordinator.set_group(&id, None);
        Ok(coordinator)
    }

    pub fn set_focused_player(
        &mut self,
        index: usize,
        updater: Option<&State<GeneralChannel<api::GroupSelectionUpdate>>>,
    ) -> Result<(), Error>{
        if index >= self.card.players(&self.available_players).len() {
            return Err(Error::CardIndexNotFound(index));
        }
        self.focused_player_index = index;
        let all_values = self.focused_player().set_all_values()?;
        self.queue_add(&all_values);
        if let Some(updater) = updater {
            updater.send(self);
        }
        Ok(())
    }

    pub fn set_group(
        &mut self,
        group_id: &str,
        updater: Option<&State<GeneralChannel<api::GroupSelectionUpdate>>>,
    ) -> Option<()> {
        let groups = self.groups();
        let ids = groups
            .iter()
            .find(|group| group.id == group_id)?
            .player_ids();
        self.card = Card::new(ids);
        if let Some(updater) = updater {
            updater.send(self);
        }

        Some(())
    }

    pub fn focused_player(&self) -> &Player {
        self.card
            .player(&self.available_players, self.focused_player_index)
            .unwrap()
    }

    pub fn amount_of_rounds(&self) -> usize {
        self.handler.amount_of_rounds()
    }

    pub fn focused_player_mut(&mut self) -> &mut Player {
        self.card
            .player_mut(&mut self.available_players, self.focused_player_index)
            .unwrap()
    }

    pub fn groups(&self) -> Vec<&dto::Group> {
        self.handler.groups().iter().collect_vec()
    }

    pub fn current_players(&self) -> Vec<&Player> {
        self.card.players(&self.available_players)
    }
}

impl FlipUpVMixCoordinator {
    fn clear_lb(idx: usize) -> Vec<VMixFunction<LeaderBoardProperty>> {
        let mut new_player = get_data::Player::null_player();

        let mut r_v: Vec<VMixFunction<LeaderBoardProperty>> = vec![];
        for i in 0..=idx {
            new_player.position = i;
            r_v.extend(new_player.set_lb());
        }
        r_v
    }

    fn queue_add<T: VMixSelectionTrait>(&self, funcs: &[VMixFunction<T>]) {
        self.queue.add(funcs)
    }

    fn set_current_through(&mut self) {
        self.current_through = self.focused_player().thru
    }

    fn make_checkin_text(&self) -> VMixFunction<LeaderBoardProperty> {
        let value = self.get_div_names()[self.selected_div_index]
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
    pub fn set_div(&mut self, div: &Division) {
        if let Some((idx, _)) = self.all_divs.iter().find_position(|d| d.id == div.id) {
            self.selected_div_index = idx;
            self.handler.set_chosen_by_ind(idx);
            self.fetch_players(false);
        }
    }

    pub fn find_division(&self, div_id: &str) -> Option<Division> {
        self.handler
            .get_divisions()
            .iter()
            .find(|div| div.id.inner() == div_id)
            .cloned()
    }

    pub fn find_player_mut(&mut self, player_id: &str) -> Option<&mut Player> {
        self.available_players.iter_mut().find(|player| player.player_id == player_id)
    }

    // TODO: Use division to set the leaderboard
    pub fn set_leaderboard(&mut self, division: &Division, lb_start_ind: Option<usize>) {
        self.queue_add(&FlipUpVMixCoordinator::clear_lb(10));
        println!("set_leaderboard");
        //let mut lb_copy = self.clone();
        self.set_current_through();
        println!("past set_lb_thru");
        if self.current_hole() <= 19 {
            println!("hole <= 19");
            self.lb_div_ind = self.selected_div_index;
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

    pub fn set_to_hole(&mut self, hole: usize) -> Result<(), Error> {
        let player = self.focused_player_mut();
        let mut actions = vec![];
        // Previously had shift-scores here
        for x in 1..=hole {
            player.hole_shown_up_until = x;
            actions.extend(player.set_hole_score()?);
        }

        self.queue_add(&actions);
        Ok(())
    }

    pub fn set_round(&mut self, idx: usize) {
        self.round_ind = idx;
        self.current_through = 0;
        let actions = self.focused_player_mut().set_round(idx);
        self.queue_add(&actions);
    }

    pub async fn fetch_event(&mut self) {}

    pub fn increase_score(&mut self, hole_update: &GeneralChannel<HoleUpdate>) -> Result<(), Error> {
        let player = self.focused_player_mut();
        if player.hole_shown_up_until <= 17 {
            let funcs = {
                // Previously had shift-scores here
                let mut funcs = player.set_hole_score()?;
                funcs.extend(player.hide_pos());
                funcs
            };
            self.queue_add(&funcs);
        }
        hole_update.send(self);
        Ok(())
    }

    pub fn hide_pos(&mut self) {
        let f = self.focused_player_mut().hide_pos();
        self.queue_add(&f)
    }

    pub fn hide_all_pos(&mut self) {
        let out = self.focused_player_mut().hide_pos();
        self.queue_add(&out);
    }

    pub fn ob_anim(&mut self) -> Result<(), Error> {
        println!("ob_anim");
        self.focused_player_mut().ob = true;
        let score = self.focused_player_mut().get_current_shown_score()?;
        self.queue_add(&score.play_mov_vmix(self.focused_player_index, true));
        Ok(())
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

    pub fn revert_score(&mut self) {
        let f = self.focused_player_mut().revert_hole_score();
        self.queue_add(&f);
    }
    pub fn reset_score(&mut self) {
        self.current_through = 0;
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

        for div in self.handler.clone().get_divisions() {
            return_vec.push(div.name.clone());
        }
        return_vec
    }

    pub fn fetch_players(&mut self, lb: bool) {
        self.available_players = self.handler.clone().get_players();
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

    pub fn make_separate_lb(&mut self, div: &Division) -> Result<(), Error> {
        if self.current_through != 0 {
            let mut new = self.clone();
            new.set_div(div);
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
            new.focused_player_index = 0;
            new.set_round(self.round_ind);
            if self.current_through > 0 {
                new.set_to_hole(self.current_through - 1)?;
            } else {
                new.set_to_hole(0)?;
            }
            new.set_leaderboard(div, None);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! Tests need to run with high node version otherwise it fails!

    use super::*;
    use crate::dto::CoordinatorBuilder;

    async fn generate_app() -> FlipUpVMixCoordinator {
        let mut app = CoordinatorBuilder::new(
            "10.170.120.134".to_string(),
            "d8f93dfb-f560-4f6c-b7a8-356164b9e4be".to_string(),
        )
        .into_coordinator()
        .await
        .unwrap();
        app.set_div(0);
        app.fetch_players(false);
        let players = app.get_player_ids();
        players
            .iter()
            .enumerate()
            .take(4 + 1)
            .skip(1)
            .for_each(|(i, player)| {
                app.set_player(player);
                //send(&handle_js_vec(test));
            });
        app.set_focused_player(1, None);
        app
    }
}
