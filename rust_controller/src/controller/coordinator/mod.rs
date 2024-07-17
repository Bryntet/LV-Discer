mod simple_queries;
mod vmix_calls;

pub use super::*;
use crate::api::{Error, GeneralChannel, HoleUpdate};
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
    round_ind: usize,
    lb_div_ind: usize,
    current_through: u8,
    pub queue: Arc<Queue>,
    card: Card,
}
#[derive(Clone, Debug)]
struct Card {
    player_ids: Vec<String>,
}
impl Card {
    fn new(player_ids: Vec<String>) -> Self {
        Self { player_ids }
    }
}

impl Card {
    fn player<'a>(&self, index: usize, players: Vec<&'a Player>) -> Option<&'a Player> {
        let player_id = self.player_ids.get(index)?;
        players
            .into_iter()
            .find(|player| &player.player_id == player_id)
    }

    fn focused_id(&self, index: usize) -> &str {
        self.player_ids.get(index).unwrap()
    }

    fn players<'a>(&self, players: Vec<&'a Player>) -> Vec<&'a Player> {
        players
            .into_iter()
            .filter(|player| self.player_ids.contains(&player.player_id))
            .collect_vec()
    }

    fn player_mut<'a>(
        &'a self,
        all_players: Vec<&'a mut Player>,
        index: usize,
    ) -> Option<&'a mut Player> {
        let player_id = self.player_ids.get(index)?;
        all_players
            .into_iter()
            .find(|player| &player.player_id == player_id)
    }
}

impl FlipUpVMixCoordinator {
    pub async fn new(ip: String, event_id: String, focused_player: usize) -> Result<Self, Error> {
        let queue = Queue::new(ip.clone())?;
        let handler = RustHandler::new(&event_id).await?;

        let available_players = handler.get_players();

        let first_group = handler.groups.first().unwrap().first().unwrap();
        let group_id = first_group.id.to_owned();
        let mut coordinator = FlipUpVMixCoordinator {
            all_divs: vec![],
            selected_div_index: 0,
            focused_player_index: focused_player,
            ip,
            card: Card::new(first_group.player_ids()),
            handler,
            round_ind: 0,
            lb_div_ind: 0,
            current_through: 0,
            queue: Arc::new(queue),
            leaderboard: Leaderboard::default(),
        };
        coordinator.queue_add(&coordinator.focused_player().set_name());
        coordinator.reset_score();
        coordinator.set_group(&group_id, None);
        Ok(coordinator)
    }
}
impl FlipUpVMixCoordinator {
    pub fn available_players(&self) -> Vec<&Player> {
        self.handler.get_players()
    }

    pub fn available_players_mut<'a>(&'a mut self) -> Vec<&'a mut Player> {
        self.handler.get_players_mut()
    }

    pub fn set_focused_player(
        &mut self,
        index: usize,
        updater: Option<&GeneralChannel<api::GroupSelectionUpdate>>,
    ) -> Result<(), Error> {
        if index >= self.card.players(self.available_players()).len() {
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

    pub fn amount_of_rounds(&self) -> usize {
        self.handler.amount_of_rounds()
    }

    pub fn focused_player_mut<'a>(&'a mut self) -> &'a mut Player {
        let index = self.focused_player_index;
        let id = self.card.focused_id(index).to_owned();
        self.handler.find_player_mut(id).unwrap()
    }

    pub fn groups(&self) -> Vec<&dto::Group> {
        self.handler.groups().iter().collect_vec()
    }

    pub fn current_players(&self) -> Vec<&Player> {
        self.card.players(self.available_players())
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
    pub fn find_player_mut(&mut self, player_id: String) -> Option<&mut Player> {
        self.handler.find_player_mut(player_id)
    }
    pub fn focused_player(&self) -> &Player {
        self.card
            .player(self.focused_player_index, self.available_players())
            .unwrap()
    }
}

// API Funcs
// basically leftover from WASM

impl FlipUpVMixCoordinator {
    pub fn set_div(&mut self, div: &Division) {
        if let Some((idx, _)) = self.all_divs.iter().find_position(|d| d.id == div.id) {
            self.selected_div_index = idx;
            self.handler.set_chosen_by_ind(idx);
        }
    }

    pub fn find_division(&self, div_id: &str) -> Option<Division> {
        self.handler
            .get_divisions()
            .iter()
            .find(|div| div.id.inner() == div_id)
            .cloned()
    }

    /*
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
            if let Some(pop) = lb_start_ind {
                for player in self.available_players_mut() {
                    player.position -= pop;
                }
            }

            self.leaderboard.update_players(LeaderboardState::new(
                self.round_ind,
                self.available_players().into_iter().cloned().collect_vec(),
            ));
            self.queue_add(&self.leaderboard.to_vmix_instructions());
        } else {
            println!("PANIC, hole > 18");
        }
    }*/

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

    pub fn increase_score(
        &mut self,
        hole_update: &GeneralChannel<HoleUpdate>,
    ) -> Result<(), Error> {
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
            .available_players()
            .into_iter()
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

    pub fn get_player_names(&self) -> Vec<String> {
        self.available_players()
            .into_iter()
            .map(|player| player.name.clone())
            .collect()
    }

    pub fn get_player_ids(&self) -> Vec<String> {
        self.available_players()
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

    // TODO: Refactor out into api function

    /*pub fn make_separate_lb(&mut self, div: &Division) -> Result<(), Error> {
        if self.current_through != 0 {
            let mut new = self.clone();
            new.set_div(div);
            new.available_players_mut()
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
                new.set_to_hole((self.current_through - 1).into())?;
            } else {
                new.set_to_hole(0)?;
            }
            new.set_leaderboard(div, None);
        }
        Ok(())
    }*/
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
