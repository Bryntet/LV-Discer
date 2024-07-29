mod simple_queries;
mod vmix_calls;
mod player_queue_system;

pub use super::*;
use crate::api::{Error, GeneralChannel, HoleUpdate, PlayerManagerUpdate};
use crate::controller::get_data::RustHandler;
use crate::controller::queries::Division;
use crate::{api, vmix};
use crate::{dto, flipup_vmix_controls};
use flipup_vmix_controls::LeaderBoardProperty;
use flipup_vmix_controls::{Leaderboard, LeaderboardState};
use get_data::Player;
use itertools::Itertools;
use rocket::State;
use std::sync::Arc;
use vmix::functions::VMixInterfacer;
use vmix::functions::{VMixPlayerInfo, VMixSelectionTrait};
use vmix::VMixQueue;
use player_queue_system::PlayerManager;

#[derive(Clone, Debug)]
pub struct FlipUpVMixCoordinator {
    pub all_divs: Vec<Arc<queries::Division>>,
    selected_div_index: usize,
    leaderboard: Leaderboard,
    focused_player_index: usize,
    ip: String,
    handler: RustHandler,
    round_ind: usize,
    lb_div_ind: usize,
    current_through: u8,
    pub vmix_queue: Arc<VMixQueue>,
    player_manager: PlayerManager,
    pub event_id: String,
}

impl FlipUpVMixCoordinator {
    pub async fn new(ip: String, event_id: String, focused_player: usize,round: usize) -> Result<Self, Error> {
        let queue = VMixQueue::new(ip.clone())?;
        let handler = RustHandler::new(&event_id,round).await?;
        

        let first_group = handler.groups.first().unwrap().first().unwrap();
        let mut coordinator = FlipUpVMixCoordinator {
            all_divs: handler.get_divisions(),
            selected_div_index: 0,
            focused_player_index: focused_player,
            ip,
            player_manager: PlayerManager::new(first_group.player_ids()),
            leaderboard: handler.get_previous_leaderboards(),
            handler,
            round_ind: round,
            lb_div_ind: 0,
            current_through: 0,
            vmix_queue: Arc::new(queue),
            event_id,
        };
        coordinator.handler.add_total_score_to_players();
        coordinator.queue_add(&coordinator.focused_player().set_name());
        coordinator.reset_score();
        Ok(coordinator)
    }
}
impl FlipUpVMixCoordinator {
    pub fn available_players(&self) -> Vec<&Player> {
        self.handler.get_players()
    }
    
    pub fn previous_rounds_players(&self) -> Vec<&Player> {
        self.handler.get_previous_rounds_players()
    }

    pub fn available_players_mut(&mut self) -> Vec<&mut Player> {
        self.handler.get_players_mut()
    }

    pub fn set_focused_player(
        &mut self,
        index: usize,
        updater: Option<&GeneralChannel<api::PlayerManagerUpdate>>,
    ) -> Result<(), Error> {
        if index >= self.player_manager.players(self.available_players()).len() {
            return Err(Error::CardIndexNotFound(index));
        }
        self.player_manager.set_focused_by_card_index(index)?;
        let all_values = self.focused_player().set_all_values()?;
        self.queue_add(&all_values);
        if let Some(updater) = updater {
            updater.send(self);
        }
        Ok(())
    }

    pub fn add_to_queue(&mut self, player_id: String, channel: &GeneralChannel<PlayerManagerUpdate>) {
        self.player_manager.add_to_queue(player_id);
        channel.send(self);
    }
    
    pub fn next_queued(&mut self, channel: &GeneralChannel<PlayerManagerUpdate>) -> Result<(), Error> {
        self.player_manager.next_queued();
        self.queue_add(&self.focused_player().set_all_values()?);
        channel.send(self);
        Ok(())
    }
    
    pub fn dto_players(&self) -> Vec<dto::Player> {
        self.player_manager.dto_players(self.available_players(),false)
    }
    
    pub fn dto_card(&self) -> Vec<dto::Player> {
        self.player_manager.dto_players(self.available_players(),true)
    }
    
    
    
    pub fn round_id(&self) -> &str {
        self.handler.round_id()
    }

    pub fn set_group(
        &mut self,
        group_id: &str,
        updater: Option<&State<GeneralChannel<api::PlayerManagerUpdate>>>,
    ) -> Option<()> {
        let groups = self.groups();
        let ids = groups
            .iter()
            .find(|group| group.id == group_id)?
            .player_ids();
        self.player_manager.replace(ids);
        if let Some(updater) = updater {
            updater.send(self);
        }
        Some(())
    }

    
    
    pub fn amount_of_rounds(&self) -> usize {
        self.handler.amount_of_rounds()
    }

    pub fn focused_player_mut(&mut self) -> &mut Player {
        let id = self.player_manager.player(self.available_players()).unwrap().player_id.to_owned();
        self.handler.find_player_mut(&id).unwrap()
    }

    pub fn groups(&self) -> Vec<&dto::Group> {
        self.handler.groups().iter().collect_vec()
    }

    pub fn current_players(&self) -> Vec<&Player> {
        self.player_manager.players(self.available_players())
    }
    fn clear_lb(idx: usize) -> Vec<VMixInterfacer<LeaderBoardProperty>> {
        let mut new_player = get_data::Player::null_player();

        let mut r_v: Vec<VMixInterfacer<LeaderBoardProperty>> = vec![];
        for i in 0..=idx {
            new_player.position = i;
            r_v.extend(new_player.set_lb());
        }
        r_v
    }

    fn queue_add<T: VMixSelectionTrait>(&self, funcs: &[VMixInterfacer<T>]) {
        self.vmix_queue.add(funcs)
    }

    fn set_current_through(&mut self) {
        self.current_through = self.focused_player().hole_shown_up_until as u8
    }

    fn make_checkin_text(&self) -> VMixInterfacer<LeaderBoardProperty> {
        let value = self.get_div_names()[self.selected_div_index]
            .to_string()
            .to_uppercase()
            + " "
            + "LEADERBOARD CHECK-IN";
        VMixInterfacer::set_text(
            value,
            LeaderBoardProperty::CheckinText,
        )
    }

    pub fn toggle_pos(&mut self) {
        let f = self.focused_player_mut().toggle_pos();
        self.queue_add(&f)
    }
    pub fn find_player_mut(&mut self, player_id: &str) -> Option<&mut Player> {
        self.handler.find_player_mut(player_id)
    }
    pub fn focused_player(&self) -> &Player {
        self.player_manager
            .player(self.available_players())
            .unwrap()
    }
    pub fn set_div(&mut self, div: &Division) {
        if let Some((idx, _)) = self.all_divs.iter().find_position(|d| d.id == div.id) {
            self.selected_div_index = idx;
            self.handler.set_chosen_by_ind(idx);
        }
    }

    pub fn find_division(&self, div_id: &str) -> Option<Arc<Division>> {
        self.handler
            .get_divisions()
            .iter()
            .find(|div| div.id.inner() == div_id).map(Arc::clone)
        
    }
    
    pub fn find_division_by_name(&self, div_name: &str) -> Option<Arc<Division>> {
        self.handler.get_divisions().iter().find(|div|div.name== div_name).map(Arc::clone)
    }

    pub fn set_leaderboard(&mut self, division: &Division, lb_start_ind: Option<usize>) {
        self.queue_add(&FlipUpVMixCoordinator::clear_lb(10));
        self.set_current_through();
        if self.current_hole() <= 18 {
            let current_players =
                self.available_players().into_iter().filter(|player|player.division.id==division.id).cloned().collect_vec();
            let previous = self.previous_rounds_players().into_iter().filter(|player|player.division.id==division.id).cloned().collect_vec();
            self.leaderboard.add_state(LeaderboardState::new(
                self.round_ind,
                current_players,
                previous
            ));
            
            self.leaderboard.send_to_vmix(self.vmix_queue.clone());
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
            actions.extend(player.increase_score()?);
        }

        self.queue_add(&actions);
        Ok(())
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
                let mut funcs = player.increase_score()?;
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
        let return_vec: Vec<VMixInterfacer<VMixPlayerInfo>> = vec![];
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

        app.set_div(&app.all_divs[0].clone());
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
