use std::sync::Arc;

use flipup_vmix_controls::LeaderBoardProperty;
use flipup_vmix_controls::{Leaderboard, LeaderboardState};
use itertools::Itertools;
pub use player::Player;
use player_queue_system::PlayerManager;
use rayon::prelude::*;
use rocket::yansi::Paint;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use vmix::functions::VMixInterfacer;
use vmix::functions::{VMixPlayerInfo, VMixSelectionTrait};
use vmix::VMixQueue;

pub use super::*;
use crate::api::{DivisionUpdate, Error, GeneralChannel, HoleUpdate, PlayerManagerUpdate};
use crate::controller::get_data::RustHandler;
use crate::controller::queries::Division;
use crate::dto::SimpleRound;
use crate::flipup_vmix_controls::CycledLeaderboard;
use crate::vmix::functions::Compare2x2;
use crate::{api, vmix};
use crate::{dto, flipup_vmix_controls};

pub mod leaderboard_cycle;
pub mod player;
mod player_queue_system;
mod simple_queries;
mod vmix_calls;

#[derive(Clone, Debug)]
pub struct FlipUpVMixCoordinator {
    pub all_divs: Vec<Arc<queries::Division>>,
    pub leaderboard_division: Arc<Division>,
    pub leaderboard: Leaderboard,
    focused_player_index: usize,
    ip: String,
    handler: RustHandler,
    round_ind: usize,
    current_through: u8,
    pub vmix_queue: Arc<VMixQueue>,
    player_manager: PlayerManager,
    pub event_ids: Vec<String>,
    featured_card: PlayerManager,
    featured_hole: u8,
    groups_featured_so_far: u8,
    pub leaderboard_round: usize,
    pub next_group: Arc<Mutex<String>>,
    pub broadcast_type: Arc<BroadcastType>,
}

impl FlipUpVMixCoordinator {
    pub async fn new(
        ip: String,
        event_ids: Vec<String>,
        focused_player: usize,
        round: usize,
        featured_hole: u8,
        broadcast_type: BroadcastType,
    ) -> Result<Self, Error> {
        assert!(!event_ids.is_empty());
        let queue = VMixQueue::new(ip.clone())?;
        let broadcast_type = Arc::new(broadcast_type);
        let handler = RustHandler::new(event_ids.clone(), round, broadcast_type.clone()).await?;

        let all_divs = handler.get_divisions();
        let first_group = handler
            .groups
            .get(round)
            .as_ref()
            .unwrap()
            .iter()
            .find(|group| !group.players.is_empty())
            .unwrap();
        let next_group = Arc::new(Mutex::new(first_group.id.to_owned()));

        let card_starts_at_hole = handler
            .groups
            .get(round)
            .unwrap()
            .iter()
            .filter(|group| !group.players.is_empty())
            .sorted_by_key(|group| {
                if let Some(start_time) = group.start_time {
                    start_time
                } else {
                    (group.start_at_hole % 18 + 1) as u32
                }
            })
            .collect_vec()
            .first()
            .unwrap()
            .to_owned();

        let mut coordinator = FlipUpVMixCoordinator {
            leaderboard_division: all_divs.first().unwrap().clone(),
            all_divs,
            focused_player_index: focused_player,
            ip,
            player_manager: PlayerManager::new(first_group.player_ids()),
            leaderboard: handler.get_previous_leaderboards(),
            featured_card: PlayerManager::new(card_starts_at_hole.player_ids()),
            handler,
            round_ind: round,
            current_through: 0,
            vmix_queue: Arc::new(queue),
            event_ids,
            groups_featured_so_far: 1,
            featured_hole,
            leaderboard_round: round,
            next_group,
            broadcast_type,
        };
        coordinator.handler.add_total_score_to_players();
        coordinator.vmix_function_on_card(&Player::set_name);
        coordinator.reset_score();
        Ok(coordinator)
    }
}
impl FlipUpVMixCoordinator {
    pub fn available_players(&self) -> Vec<&Player> {
        self.handler.get_players()
    }

    pub fn increase_leaderboard_skip(&mut self) {
        self.leaderboard.skip += 1;
        self.set_leaderboard(None);
    }

    pub fn decrease_leaderboard_skip(&mut self) {
        if self.leaderboard.skip > 0 {
            self.leaderboard.skip -= 1;
        }
        self.set_leaderboard(None);
    }

    pub fn reset_leaderboard_skip(&mut self) {
        self.leaderboard.skip = 0;
        self.set_leaderboard(None);
    }

    pub fn previous_rounds_players(&self) -> Vec<&Player> {
        self.handler.get_previous_rounds_players()
    }

    pub fn update_featured_card(&mut self) -> Result<(), Error> {
        self.add_state_to_leaderboard();

        let mut instructs = self
            .featured_card
            .card(self.available_players())
            .into_iter()
            .enumerate()
            .flat_map(|(i, player)| {
                player
                    .set_all_compare_2x2_values(i, &self.leaderboard, false)
                    .expect("Should work due to set all values already passing")
            })
            .collect_vec();

        self.add_null_players(&mut instructs, &self.featured_card)?;
        self.queue_add(
            instructs
                .into_iter()
                .map(VMixInterfacer::into_featured)
                .collect_vec()
                .iter(),
        );
        let featured_hole = self.featured_hole;
        let stats = self.make_stats();
        let holes = self
            .featured_card
            .player(self.available_players())
            .unwrap()
            .holes
            .clone();
        let div = &self.leaderboard_division.clone();
        let out = self
            .focused_player_mut()
            .results
            .get_hole_info(featured_hole, stats, &holes, div)
            .into_iter()
            .map(VMixInterfacer::into_featured_hole_card)
            .collect_vec();

        self.queue_add(&out);
        Ok(())
    }

    pub async fn next_featured_card(&mut self) -> Result<(), Error> {
        let next_group_id = self.next_group.lock().await.clone();
        if let Some(group) = self.groups().iter().find(|group| group.id == next_group_id) {
            self.featured_card.replace(group.player_ids());
            self.update_featured_card()?;
            self.groups_featured_so_far += 1;
        }
        Ok(())
    }

    pub fn rewind_card(&mut self) {
        if self.groups_featured_so_far < 2 {
            self.groups_featured_so_far = 0
        } else {
            self.groups_featured_so_far -= 2;
        }
        self.next_featured_card();
    }

    pub fn available_players_mut(&mut self) -> Vec<&mut Player> {
        self.handler.get_players_mut()
    }

    pub fn set_focused_player(
        &mut self,
        index: usize,
        player_updater: GeneralChannel<api::PlayerManagerUpdate>,
        division_updater: GeneralChannel<DivisionUpdate>,
    ) -> Result<(), Error> {
        if index >= self.player_manager.players(self.available_players()).len() {
            return Err(Error::CardIndexNotFound(index));
        }
        self.player_manager.set_focused_by_card_index(index)?;
        self.leaderboard_division = self.focused_player().division.clone();
        self.add_state_to_leaderboard();
        let all_values = self.focused_player().set_all_values(
            &self.leaderboard,
            match self.broadcast_type.as_ref() {
                BroadcastType::PostLive => false,
                BroadcastType::Live => true,
            },
        )?;

        let current = self
            .focused_player()
            .set_all_current_player_values(&all_values);
        self.queue_add(&all_values);
        self.queue_add(&current);
        player_updater.send_from_coordinator(self);
        division_updater.send_from_coordinator(self);
        Ok(())
    }

    pub fn add_to_queue(
        &mut self,
        player_id: String,
        hole: Option<u8>,
        throw: Option<u8>,
        channel: GeneralChannel<PlayerManagerUpdate>,
    ) {
        let broadcast_type = self.broadcast_type.clone();
        if let Some(player) = self.find_player_mut(&player_id) {
            let hole = match hole {
                Some(0) | None => match broadcast_type.as_ref() {
                    BroadcastType::PostLive => 0,
                    BroadcastType::Live => player.amount_of_holes_finished() as u8,
                },
                Some(h) => h,
            };
            player.hole_shown_up_until = hole as usize;
            if let Some(throw) = throw {
                player.throws = throw;
            }
        }

        self.player_manager.add_to_queue(player_id);
        channel.send_from_coordinator(self);
    }

    pub fn next_queued(
        &mut self,
        channel: GeneralChannel<PlayerManagerUpdate>,
    ) -> Result<(), Error> {
        self.player_manager.next_queued();
        let up_until = self.focused_player().hole_shown_up_until;
        self.focused_player_mut().total_score = 0;
        self.add_state_to_leaderboard();
        self.focused_player_mut()
            .fix_round_score(Some(up_until as u8));
        if *self.broadcast_type == BroadcastType::Live {
            if let Some(player) = self.leaderboard.get_lb_player(self.focused_player()) {
                let current_round_score = self.focused_player().round_score;
                self.focused_player_mut().total_score =
                    player.total_score - player.round_score + current_round_score;
            }
        }
        let all = self
            .focused_player()
            .set_all_values(&self.leaderboard, false)?;
        self.queue_add(&all);
        let current = self.focused_player().set_all_current_player_values(&all);
        self.queue_add(&current);
        channel.send_from_coordinator(self);
        Ok(())
    }

    pub fn dto_players(&self) -> Vec<dto::Player> {
        self.player_manager
            .dto_players(self.available_players(), false)
    }

    pub fn dto_card(&self) -> Vec<dto::Player> {
        self.player_manager
            .dto_players(self.available_players(), true)
    }

    pub fn round_ids(&self) -> Vec<String> {
        self.handler.round_ids()
    }

    pub fn set_group(
        &mut self,
        group_id: &str,
        updater: GeneralChannel<api::PlayerManagerUpdate>,
    ) -> Result<(), Error> {
        let groups = self.groups();
        let ids = groups
            .iter()
            .find(|group| group.id == group_id)
            .ok_or(Error::UnableToParse)?
            .player_ids();
        self.player_manager.replace(ids);
        self.add_state_to_leaderboard();
        let player = self.focused_player();
        let all = player.set_all_values(&self.leaderboard, false)?;
        let current = player.set_all_current_player_values(&all);
        self.queue_add(&all);
        self.queue_add(&current);

        updater.send_from_coordinator(self);

        Ok(())
    }

    pub fn update_group_to_focused_player_group(
        &mut self,
        player_updater: GeneralChannel<PlayerManagerUpdate>,
    ) -> Result<(), Error> {
        let focused_player_id = self.focused_player().player_id.to_owned();
        let group = self
            .groups()
            .into_par_iter()
            .find_first(|group| group.player_ids().iter().contains(&focused_player_id))
            .expect("Player needs to be in group");

        let group_id = group.id.to_owned();
        self.set_group(&group_id, player_updater.clone())?;

        self.player_manager.set_focused(&focused_player_id);
        player_updater.send_from_coordinator(self);
        let mut compare_2x2 = self
            .player_manager
            .card(self.available_players())
            .into_par_iter()
            .enumerate()
            .flat_map(|(index, player)| {
                player
                    .set_all_compare_2x2_values(index, &self.leaderboard, false)
                    .expect("Should work due to set all values already passing")
            })
            .collect::<Vec<_>>();

        self.add_null_players(&mut compare_2x2, &self.player_manager)?;

        self.queue_add(&compare_2x2);
        Ok(())
    }

    pub fn add_null_players(
        &self,
        instructions: &mut Vec<VMixInterfacer<Compare2x2>>,
        player_manager: &PlayerManager,
    ) -> Result<(), Error> {
        for index in player_manager.players(self.available_players()).len()..4 {
            instructions.extend(Player::null_player().set_all_compare_2x2_values(
                index,
                &self.leaderboard,
                true,
            )?)
        }
        Ok(())
    }

    pub fn dto_rounds(&self) -> Vec<dto::SimpleRound> {
        let mut round_ids: Vec<Vec<String>> = vec![];
        self.handler.round_ids.iter().for_each(|ids| {
            ids.iter()
                .enumerate()
                .for_each(|(round, round_id)| match round_ids.get_mut(round) {
                    Some(rounds) => rounds.push(round_id.to_owned()),
                    None => round_ids.push(vec![round_id.to_owned()]),
                })
        });
        round_ids
            .into_iter()
            .enumerate()
            .map(|(round, id)| SimpleRound::new(round, id, round == self.leaderboard_round))
            .collect()
    }
    pub fn amount_of_rounds(&self) -> usize {
        self.handler.amount_of_rounds()
    }

    pub fn focused_player_mut(&mut self) -> &mut Player {
        let id = self
            .player_manager
            .player(self.available_players())
            .unwrap_or(self.available_players().first().unwrap())
            .player_id
            .to_owned();
        self.handler.find_player_mut(&id).unwrap()
    }

    pub fn groups(&self) -> &Vec<dto::Group> {
        self.handler.groups()
    }

    pub fn current_players(&self) -> Vec<&Player> {
        self.player_manager.players(self.available_players())
    }

    pub fn vmix_function_on_card<T: VMixSelectionTrait + Sized>(
        &self,
        func: &dyn Fn(&Player) -> Vec<VMixInterfacer<T>>,
    ) {
        let current_players = self.current_players();
        self.vmix_queue
            .add(current_players.into_iter().flat_map(func))
    }
    pub fn vmix_function_on_card_mut<T: VMixSelectionTrait + Sized>(
        &mut self,
        func: &dyn Fn(&mut Player) -> Vec<VMixInterfacer<T>>,
    ) {
        let current_players = self
            .current_players()
            .iter()
            .map(|player| player.player_id.to_owned())
            .collect_vec();
        let mut players = self.handler.get_players_mut();
        let mut funcs = vec![];
        for player_id in current_players {
            if let Some(player) = players
                .par_iter_mut()
                .find_any(|player| player.player_id == player_id)
            {
                funcs.push(func(player))
            }
        }
        self.vmix_queue
            .add(funcs.into_iter().flat_map(|fun| fun.into_iter()))
    }

    fn clear_lb(idx: usize) -> Vec<VMixInterfacer<LeaderBoardProperty>> {
        let mut new_player = Player::null_player();

        let mut r_v: Vec<VMixInterfacer<LeaderBoardProperty>> = vec![];
        for i in 0..=idx {
            new_player.position = i;
            r_v.extend(new_player.set_lb());
        }
        r_v
    }

    pub fn clear_little_cycling_lb() -> Vec<VMixInterfacer<CycledLeaderboard>> {
        FlipUpVMixCoordinator::clear_lb(6)
            .into_iter()
            .filter_map(|interface| {
                Some(VMixInterfacer::<CycledLeaderboard>::from(
                    interface.to_top_6()?,
                ))
            })
            .collect()
    }

    fn queue_add<'a, T: VMixSelectionTrait + 'a>(
        &self,
        funcs: impl IntoIterator<Item = &'a VMixInterfacer<T>>,
    ) {
        self.vmix_queue.add_ref(funcs.into_iter())
    }

    fn set_current_through(&mut self) {
        self.current_through = self.focused_player().hole_shown_up_until as u8
    }

    pub fn find_player_mut(&mut self, player_id: &str) -> Option<&mut Player> {
        self.handler.find_player_mut(player_id)
    }
    pub fn focused_player(&self) -> &Player {
        self.player_manager
            .player(self.available_players())
            .unwrap()
    }
    pub fn set_div(&mut self, div: &Division, channel: GeneralChannel<DivisionUpdate>) {
        if let Some(div) = self.all_divs.iter().find(|d| d.id == div.id) {
            self.leaderboard_division = dbg!(div.clone());
        }
        channel.send_from_coordinator(self);
    }

    pub fn find_division(&self, div_id: &str) -> Option<Arc<Division>> {
        self.handler
            .get_divisions()
            .iter()
            .find(|div| div.id.inner() == div_id)
            .map(Arc::clone)
    }

    pub fn find_division_by_name(&self, div_name: &str) -> Option<Arc<Division>> {
        self.handler
            .get_divisions()
            .iter()
            .find(|div| div.name == div_name)
            .map(Arc::clone)
    }

    pub fn current_leaderboard_state(&self) -> LeaderboardState {
        let current_players = self.available_players().into_iter().cloned().collect_vec();
        let previous = self
            .previous_rounds_players()
            .into_iter()
            .cloned()
            .collect_vec();
        LeaderboardState::new(self.round_ind, current_players, previous)
    }

    pub fn add_state_to_leaderboard(&mut self) {
        self.set_current_through();
        let current_players = self.available_players().into_iter().cloned().collect_vec();
        let previous = self
            .previous_rounds_players()
            .into_iter()
            .cloned()
            .collect_vec();
        self.leaderboard.add_state(LeaderboardState::new(
            self.round_ind,
            current_players,
            previous,
        ));

        let lb_players = self
            .available_players()
            .into_iter()
            .flat_map(|player| self.leaderboard.get_lb_player(player))
            .collect_vec();
        let mut all_players = self.available_players_mut();
        for lb_player in lb_players {
            if let Some(player) = all_players
                .iter_mut()
                .find(|player| player.player_id == lb_player.id)
            {
                player.total_score = lb_player.total_score
            }
        }
    }

    pub fn set_leaderboard(&mut self, lb_start_ind: Option<usize>) {
        if self.current_hole() <= 18 {
            self.add_state_to_leaderboard();
            self.queue_add(&FlipUpVMixCoordinator::clear_lb(10));
            self.leaderboard.send_to_vmix(
                &self.leaderboard_division,
                self.vmix_queue.clone(),
                self.leaderboard_round,
            );
        } else {
            println!("PANIC, hole > 18");
        }
    }

    pub fn set_to_hole(&mut self, hole: usize) -> Result<(), Error> {
        let player = self.focused_player_mut();
        let mut player_interfaces = vec![];
        let mut current_player_interfaces = vec![];
        // Previously had shift-scores here
        for x in 1..=hole {
            player.hole_shown_up_until = x;
            let all_values = player.increase_score()?;
            let current = player.set_all_current_player_values(&all_values);
            player_interfaces.extend(all_values);
            current_player_interfaces.extend(current);
        }

        self.queue_add(&player_interfaces);
        self.queue_add(&current_player_interfaces);
        Ok(())
    }

    pub async fn fetch_event(&mut self) {}

    pub fn increase_score(
        &mut self,
        hole_update: &GeneralChannel<HoleUpdate>,
    ) -> Result<(), Error> {
        let broadcast_type = self.broadcast_type.clone();
        let player = self.focused_player_mut();

        dbg!(&player.name);
        let throws_condition = match broadcast_type.as_ref() {
            BroadcastType::Live => player.throws != 0,
            BroadcastType::PostLive => true,
        };

        if throws_condition && player.hole_shown_up_until <= 17 {
            let mut f = player.increase_score()?;
            self.add_state_to_leaderboard();
            let player = self.focused_player();
            let lb_things = player.add_lb_things(&self.leaderboard);

            let mut current = player.set_all_current_player_values(&f);
            let more_current = lb_things
                .iter()
                .flat_map(|interface| interface.to_owned().into_current_player())
                .collect_vec();
            current.extend(more_current);
            f.extend(lb_things);
            self.queue_add(&f);
            self.queue_add(&current);
        }
        hole_update.send_from_coordinator(self);
        Ok(())
    }

    pub fn ob_anim(&mut self) -> Result<(), Error> {
        let score = self.focused_player_mut().get_current_shown_score();
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
        let player = self.focused_player_mut();

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

        match self.broadcast_type.as_ref() {
            BroadcastType::Live => {
                let f = self.focused_player_mut().reset_scores();
                self.queue_add(&f)
            }
            BroadcastType::PostLive => self.vmix_function_on_card_mut(&Player::reset_scores),
        }
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
        self.queue_add(&self.focused_player().set_all_current_player_values(&f));
        self.queue_add(&f)
    }

    pub fn decrease_throw(&mut self) {
        if self.focused_player().throws != 0 {
            self.focused_player_mut().throws -= 1;
            let f = &[self.focused_player_mut().set_throw()];
            self.queue_add(&self.focused_player().set_all_current_player_values(f));
            self.queue_add(f);
        }
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

use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
#[derive(Debug, Deserialize, JsonSchema, FromFormField, Default, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastType {
    Live,
    #[default]
    PostLive,
}
#[cfg(test)]
mod tests {
    //! Tests need to run with high node version otherwise it fails!
}
