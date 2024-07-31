use itertools::Itertools;

use crate::api::Error;
use crate::controller::Player;
use crate::dto;

#[derive(Clone, Debug)]
pub struct PlayerWithQueue {
    inside_card: bool,
    player_id: String,
    queue_position: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct PlayerManager {
    managed_players: Vec<PlayerWithQueue>,
    focused: usize,
}

fn new_card(card: Vec<String>) -> Vec<PlayerWithQueue> {
    card.into_iter()
        .map(|id| PlayerWithQueue {
            inside_card: true,
            player_id: id,
            queue_position: None,
        })
        .collect_vec()
}
impl PlayerManager {
    pub fn new(card: Vec<String>) -> Self {
        Self {
            managed_players: new_card(card),
            focused: 0,
        }
    }
    pub fn replace(&mut self, new_card_ids: Vec<String>) {
        let mut new_card = new_card(new_card_ids)
            .into_iter()
            .map(|player| {
                if let Some(current) = self
                    .managed_players
                    .iter()
                    .find(|p| p.player_id == player.player_id)
                {
                    let mut player = current.to_owned();
                    player.inside_card = true;
                    player
                } else {
                    player
                }
            })
            .collect_vec();

        let mut new_whole_list = self
            .managed_players
            .iter()
            .filter(|player| {
                if new_card
                    .iter()
                    .map(|p| &p.player_id)
                    .contains(&player.player_id)
                {
                    false
                } else {
                    player.queue_position.is_some()
                }
            })
            .cloned()
            .map(|mut player| {
                player.inside_card = false;
                player
            })
            .rev()
            .collect_vec();
        new_card.reverse();
        new_whole_list.extend(new_card);
        new_whole_list.reverse();
        self.managed_players = new_whole_list;
        if self.focused >= self.managed_players.len() {
            self.focused = self.managed_players.len() - 1;
        }
    }
    pub fn player<'a>(&self, players: Vec<&'a Player>) -> Option<&'a Player> {
        let queue = &self.managed_players[self.focused];
        players
            .into_iter()
            .find(|player| player.player_id == queue.player_id)
    }

    pub fn player_mut<'a>(&'a self, players: Vec<&'a mut Player>) -> Option<&'a mut Player> {
        let queue = &self.managed_players[self.focused];
        players
            .into_iter()
            .find(|player| player.player_id == queue.player_id)
    }

    fn internal_card(&self) -> Vec<&PlayerWithQueue> {
        self.managed_players
            .iter()
            .filter(|queue| queue.inside_card)
            .collect_vec()
    }

    pub fn card<'a>(&self, players: Vec<&'a Player>) -> Vec<&'a Player> {
        let ids = self
            .internal_card()
            .into_iter()
            .map(|player| player.player_id.to_owned())
            .collect_vec();
        players
            .into_iter()
            .filter(|player| ids.contains(&player.player_id))
            .collect_vec()
    }

    fn get_card_index(&self, index: usize) -> Option<&PlayerWithQueue> {
        let card = self.internal_card();
        card.get(index).copied()
    }

    fn focused_player(&self) -> &PlayerWithQueue {
        self.managed_players
            .get(self.focused)
            .expect("We need to have a focused player")
    }

    pub fn focused_id(&self, index: usize) -> Option<&str> {
        Some(&self.get_card_index(index)?.player_id)
    }

    fn next_queue_position(&self) -> usize {
        self.managed_players
            .iter()
            .flat_map(|player| player.queue_position)
            .max()
            .unwrap_or(0)
            + 1
    }

    // Remember to send channel update when using this func
    pub fn add_to_queue(&mut self, id: String) {
        let next_queue_pos = self.next_queue_position();
        if let Some(player) = self
            .managed_players
            .iter_mut()
            .find(|player| player.player_id == id)
        {
            if player.queue_position.is_none() {
                player.queue_position = Some(next_queue_pos);
            }
        } else {
            self.managed_players.push(PlayerWithQueue {
                inside_card: false,
                player_id: id,
                queue_position: Some(next_queue_pos),
            })
        }
    }

    fn clear_all_empty(&mut self) {
        let current_focused = self.focused_player().player_id.clone();
        self.managed_players = self
            .managed_players
            .iter()
            .filter(|player| {
                player.inside_card
                    || player.queue_position.is_some()
                    || player.player_id == current_focused
            })
            .cloned()
            .collect_vec();
        self.set_focused(&current_focused);
    }

    // Remember to send channel update when using this func
    pub fn next_queued(&mut self) {
        let mut focused_player_id = None;
        self.managed_players.iter_mut().for_each(|queue| {
            let pos = &mut queue.queue_position;
            if pos.is_some_and(|num| num == 1) {
                *pos = None;
                focused_player_id = Some(queue.player_id.clone())
            } else if let Some(pos) = pos {
                *pos -= 1;
            }
        });
        if let Some(id) = focused_player_id {
            self.set_focused(&id)
        }
        self.clear_all_empty();
    }

    fn set_focused(&mut self, id: &str) {
        self.managed_players
            .iter()
            .enumerate()
            .for_each(|(i, player)| {
                if player.player_id == id {
                    self.focused = i;
                }
            })
    }

    pub fn set_focused_by_card_index(&mut self, index: usize) -> Result<(), Error> {
        let card = self.internal_card();
        let focused_player = card.get(index).ok_or(Error::PlayerInCardNotFound(index))?;
        self.set_focused(&focused_player.player_id.to_owned());
        Ok(())
    }

    pub fn queued_players(&self) -> Vec<&PlayerWithQueue> {
        self.managed_players
            .iter()
            .filter(|player| player.queue_position.is_some())
            .collect_vec()
    }

    pub fn players<'a>(&self, players: Vec<&'a Player>) -> Vec<&'a Player> {
        // This list has to be done like this to make sure it's sorted correctly.
        let mut out_players = vec![];
        for player in &self.managed_players {
            if let Some(player) = players
                .iter()
                .find(|other_player| player.player_id == other_player.player_id)
            {
                out_players.push(*player);
            }
        }
        out_players
    }

    pub fn dto_players(&self, players: Vec<&Player>, card_only: bool) -> Vec<dto::Player> {
        let mut dto_players = vec![];
        for (i, player) in self
            .managed_players
            .iter()
            .enumerate()
            .filter(|(_, player)| if card_only { player.inside_card } else { true })
        {
            if let Some(normal_player) = players
                .iter()
                .find(|normal_player| player.player_id == normal_player.player_id)
            {
                let mut dto = dto::Player::from(*normal_player);
                if self.focused == i {
                    dto.focused = true;
                }
                dto.queue = player.queue_position;
                dto_players.push(dto);
            }
        }
        dto_players
    }
}
