use crate::controller::Player;
use crate::vmix::functions::VMixFunction;
use itertools::Itertools;
#[derive(Debug, Clone, Default)]
pub struct Leaderboard {
    states: Vec<LeaderboardState>,
}
#[derive(Debug, Clone)]
pub struct LeaderboardState {
    hole: usize,
    round: usize,
    players: Vec<Player>,
}

impl Leaderboard {
    pub fn new(state: LeaderboardState) -> Self {
        Self {
            states: vec![state],
        }
    }

    fn current_state(&self) -> &LeaderboardState {
        self.states
            .last()
            .expect("Leaderboard has to have state when initiated")
    }

    /// Returns the previous state of the leaderboard if it exists
    fn previous_state(&self) -> Option<&LeaderboardState> {
        if self.states.len() < 2 {
            return None;
        }
        self.states.get(self.states.len() - 2)
    }

    pub fn update_players(&mut self, new_state: LeaderboardState) {
        if let Some(state) = self
            .states
            .iter_mut()
            .find(|state| state.round == new_state.round)
        {
            *state = new_state;
        } else {
            self.states.push(new_state);
        }
    }

    pub fn to_vmix_instructions(&self) -> Vec<VMixFunction<LeaderBoardProperty>> {
        self.current_state()
            .to_vmix_instructions(self.previous_state())
    }
}
impl LeaderboardState {
    pub fn new(round: usize, mut players: Vec<Player>) -> Self {
        let hole = players.iter().map(|p| p.hole).max().expect("Vec not empty");
        Self::sort_players(&mut players);
        Self {
            hole,
            round,
            players,
        }
    }
    fn sort_players(players: &mut [Player]) {
        players.sort_by(|player_a, player_b| player_a.total_score.cmp(&player_b.total_score))
    }

    fn leaderboard_players(&self, other: Option<&Self>) -> Vec<LeaderboardPlayer> {
        let max_score = self
            .players
            .iter()
            .map(|p| p.round_score)
            .max()
            .unwrap_or_default();
        let other = other.map(|lb| lb.players.iter().collect_vec());
        let players_with_pos = Self::players_with_positions(self.players.iter().collect_vec());
        players_with_pos
            .into_iter()
            .enumerate()
            .map(|(real_pos, (index, player))| {
                LeaderboardPlayer::new(player, real_pos, index, max_score, other.as_ref())
            })
            .collect_vec()
    }

    fn players_with_positions(players: Vec<&Player>) -> Vec<(usize, &Player)> {
        let mut pos = 1;
        let mut skipped = 0;
        let mut last_score = players.first().map(|p| p.total_score).unwrap_or_default();
        players
            .into_iter()
            .map(|player| {
                if player.total_score != last_score {
                    pos += skipped + 1;
                    skipped = 0;
                } else {
                    skipped += 1;
                }
                last_score = player.total_score;
                (pos, player)
            })
            .collect_vec()
    }

    pub fn to_vmix_instructions(
        &self,
        other: Option<&Self>,
    ) -> Vec<VMixFunction<LeaderBoardProperty>> {
        let players = self.leaderboard_players(other);
        players
            .iter()
            .map(LeaderboardPlayer::combine)
            .flatten()
            .collect_vec()
    }
}

struct LeaderboardPlayer {
    position: usize,
    index: usize,
    movement: LeaderboardMovement,
    hot_round: bool,
    name: String,
    round_score: isize,
    total_score: isize,
    thru: usize,
}

impl LeaderboardPlayer {
    /// Creates a new LeaderboardPlayer
    ///
    /// # Arguments
    ///
    /// * `player` - The player to create the leaderboard player from
    ///
    /// * `pos` - The position of the player in the leaderboard
    ///
    /// * `max_score_reached` - The maximum score reached by any player in the round
    ///
    /// * `other_board` - The other leaderboard to compare the movement to
    pub fn new(
        player: &Player,
        pos: usize,
        index: usize,
        max_score_reached: isize,
        other_board: Option<&Vec<&Player>>,
    ) -> Self {
        let other_pos = other_board
            .and_then(|players| {
                players
                    .iter()
                    .enumerate()
                    .find(|(_, other_player)| other_player.name == player.name)
            })
            .map(|(pos, _)| pos);
        let movement = match other_pos {
            Some(other_pos) => LeaderboardMovement::new(pos, other_pos),
            None => LeaderboardMovement::Same,
        };
        LeaderboardPlayer {
            position: pos,
            index,
            movement,
            hot_round: player.round_score == max_score_reached,
            name: player.name.clone(),
            round_score: player.round_score,
            total_score: player.total_score,
            thru: player.thru,
        }
    }

    fn set_hot_round(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetImage {
            value: if self.hot_round {
                Image::Flames
            } else {
                Image::Nothing
            }
            .to_location(),
            input: LeaderBoardProperty::HotRound(self.position).into(),
        }
    }

    fn set_round_score(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.round_score.to_string(),
            input: LeaderBoardProperty::RoundScore(self.position).into(),
        }
    }

    fn set_total_score(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.total_score.to_string(),
            input: LeaderBoardProperty::TotalScore { pos: self.position }.into(),
        }
    }

    fn set_position(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.position.to_string(),
            input: LeaderBoardProperty::Position {
                pos: self.position,
                lb_pos: self.position,
                tied: false,
            }
            .into(),
        }
    }

    fn set_movement_img(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetImage {
            value: match self.movement {
                LeaderboardMovement::Up(_) => Image::RedTriDown,
                LeaderboardMovement::Down(_) => Image::GreenTriUp,
                LeaderboardMovement::Same => Image::Nothing,
            }
            .to_location(),
            input: LeaderBoardProperty::Arrow { pos: self.position }.into(),
        }
    }

    fn set_movement_text(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: match self.movement {
                LeaderboardMovement::Up(n) => format!("+{}", n),
                LeaderboardMovement::Down(n) => format!("-{}", n),
                LeaderboardMovement::Same => "".to_string(),
            },
            input: LeaderBoardProperty::Move { pos: self.position }.into(),
        }
    }

    fn set_thru(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.thru.to_string(),
            input: LeaderBoardProperty::Thru(self.position).into(),
        }
    }

    fn set_name(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.name.clone(),
            input: LeaderBoardProperty::Name(self.position).into(),
        }
    }

    pub fn combine(&self) -> Vec<VMixFunction<LeaderBoardProperty>> {
        vec![
            self.set_hot_round(),
            self.set_round_score(),
            self.set_total_score(),
            self.set_position(),
            self.set_movement_img(),
            self.set_movement_text(),
            self.set_thru(),
            self.set_name(),
        ]
    }
}
enum LeaderboardMovement {
    Up(usize),
    Down(usize),
    Same,
}

impl LeaderboardMovement {
    pub fn new(pos: usize, other_pos: usize) -> Self {
        match pos.cmp(&other_pos) {
            std::cmp::Ordering::Less => Self::Down(other_pos - pos),
            std::cmp::Ordering::Greater => Self::Up(pos - other_pos),
            std::cmp::Ordering::Equal => Self::Same,
        }
    }
}

mod prop {
    use crate::vmix::functions::{VMixSelection, VMixSelectionTrait};

    pub enum LeaderBoardProperty {
        Position {
            pos: usize,
            lb_pos: usize,
            tied: bool,
        },
        Name(usize),
        HotRound(usize),
        RoundScore(usize),
        TotalScore {
            pos: usize,
        },
        Move {
            pos: usize,
        },
        Arrow {
            pos: usize,
        },
        Thru(usize),
        CheckinText,
        TotalScoreTitle,
    }
    impl VMixSelectionTrait for LeaderBoardProperty {
        fn get_selection(&self) -> String {
            self.get_id()
                + &(match self {
                    LeaderBoardProperty::Position { pos, lb_pos, tied } => {
                        if *tied && lb_pos != &0 {
                            format!("SelectedName=pos#{}.Text&Value=T{}", pos, lb_pos)
                        } else {
                            format!(
                                "SelectedName=pos#{}.Text&Value={}",
                                pos,
                                if lb_pos != &0 {
                                    lb_pos.to_string()
                                } else {
                                    "".to_string()
                                }
                            )
                        }
                    }
                    LeaderBoardProperty::Name(pos) => format!("SelectedName=name#{}.Text", pos),
                    LeaderBoardProperty::HotRound(pos) => format!("SelectedName=hrp{}.Source", pos),
                    LeaderBoardProperty::RoundScore(pos) => format!("SelectedName=rs#{}.Text", pos),
                    LeaderBoardProperty::TotalScore { pos, .. } => {
                        format!("SelectedName=ts#{}.Text", pos)
                    }
                    LeaderBoardProperty::TotalScoreTitle => "SelectedName=ts.Text".to_string(),
                    LeaderBoardProperty::Move { pos, .. } => {
                        format!("SelectedName=move{}.Text", pos)
                    }
                    LeaderBoardProperty::Arrow { pos, .. } => {
                        format!("SelectedName=arw{}.Source", pos)
                    }
                    LeaderBoardProperty::Thru(pos) => format!("SelectedName=thru#{}.Text", pos),
                    LeaderBoardProperty::CheckinText => "SelectedName=checkintext.Text".to_string(),
                })
        }

        fn get_id(&self) -> String {
            "Input=0e76d38f-6e8d-4f7d-b1a6-e76f695f2094&".to_string()
        }
    }

    impl From<LeaderBoardProperty> for VMixSelection<LeaderBoardProperty> {
        fn from(prop: LeaderBoardProperty) -> Self {
            VMixSelection(prop)
        }
    }
}
use crate::flipup_vmix_controls::Image;
pub use prop::LeaderBoardProperty;

#[cfg(test)]
mod test {
    use super::{Leaderboard, LeaderboardState};
    use crate::get_data::Player;
    use crate::vmix::functions::VMixFunction;
    use itertools::Itertools;

    #[test]
    fn test() {
        let p = LeaderboardState::new(1, vec![Player::default()]);
        let mut a = Leaderboard::new(p.clone());
        dbg!(a
            .to_vmix_instructions()
            .iter()
            .map(|a| a.to_cmd())
            .collect_vec());
        panic!()
    }
}
