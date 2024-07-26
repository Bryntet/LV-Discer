use crate::controller::Player;
use crate::vmix::functions::VMixFunction;
use itertools::Itertools;
#[derive(Debug, Clone, Default)]
pub struct Leaderboard {
    states: Vec<LeaderboardState>,
}
#[derive(Debug, Clone)]
pub struct LeaderboardState {
    where_to_start: LeaderboardStart,
    round: usize,
    players: Vec<Player>,
}
#[derive(Debug, Clone)]
pub enum LeaderboardStart {
    Latest,
    Specific(u8),
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
        
        players
            .iter_mut()
            .for_each(|player| player.fix_round_score(None));
        Self::sort_players(&mut players);
        Self {
            where_to_start: LeaderboardStart::Latest,
            round,
            players,
        }
    }
}

impl LeaderboardState {
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
                LeaderboardPlayer::new(
                    player,
                    real_pos + 1,
                    max_score,
                    other.as_ref(),
                    self.round,
                    &self.players,
                )
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
                    dbg!("hi");
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
            .flat_map(LeaderboardPlayer::combine)
            .collect_vec()
    }
}

#[derive(Debug)]
struct LeaderboardPlayer {
    index: usize,
    position: usize,
    movement: LeaderboardMovement,
    hot_round: bool,
    name: String,
    round_score: isize,
    total_score: isize,
    thru: u8,
    tied: Option<u8>,
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
        index: usize,
        max_score_reached: isize,
        other_board: Option<&Vec<&Player>>,
        round: usize,
        all_other_players: &[Player],
    ) -> Self {
        let pos = all_other_players
            .iter()
            .filter(|other_player| other_player.total_score < player.total_score)
            .count()
            + 1;
        let other_pos = other_board
            .and_then(|players| {
                players
                    .iter()
                    .enumerate()
                    .find(|(_, other_player)| other_player.player_id == player.player_id)
            })
            .map(|(pos, _)| pos);
        let movement = match other_pos {
            Some(other_pos) => LeaderboardMovement::new(index, other_pos),
            None => LeaderboardMovement::Same,
        };
        let tie = {
            let tie_count = all_other_players
                .iter()
                .filter(|lb_player| lb_player.total_score == player.total_score)
                .count();
            if tie_count > 1 {
                Some(tie_count as u8)
            } else {
                None
            }
        };

        LeaderboardPlayer {
            index,
            position: pos,
            movement,
            hot_round: player.round_score == max_score_reached && round != 1,
            name: player.name.clone(),
            round_score: player.round_score,
            total_score: player.total_score,
            thru: player.hole_shown_up_until as u8,
            tied: tie,
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
            input: LeaderBoardProperty::HotRound(self.index).into(),
        }
    }

    fn set_round_score(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.round_score.to_string(),
            input: LeaderBoardProperty::RoundScore(self.index).into(),
        }
    }

    fn set_total_score(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.total_score.to_string(),
            input: LeaderBoardProperty::TotalScore { pos: self.index }.into(),
        }
    }

    fn set_position(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: "".to_string(),
            input: LeaderBoardProperty::Position {
                pos: self.index,
                lb_pos: self.position,
                tied: self.tied,
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
            input: LeaderBoardProperty::Arrow { pos: self.index }.into(),
        }
    }

    fn set_movement_text(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: match self.movement {
                LeaderboardMovement::Up(n) => format!("+{}", n),
                LeaderboardMovement::Down(n) => format!("-{}", n),
                LeaderboardMovement::Same => " ".to_string(),
            },
            input: LeaderBoardProperty::Move { pos: self.index }.into(),
        }
    }

    fn set_thru(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.thru.to_string(),
            input: LeaderBoardProperty::Thru(self.index).into(),
        }
    }

    fn set_name(&self) -> VMixFunction<LeaderBoardProperty> {
        VMixFunction::SetText {
            value: self.name.clone(),
            input: LeaderBoardProperty::Name(self.index).into(),
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
#[derive(Debug)]
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
            tied: Option<u8>,
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
        fn get_selection_name(&self) -> String {
            match self {
                LeaderBoardProperty::Position { pos, .. } => {
                    format!("pos#{}", pos)
                }
                LeaderBoardProperty::Name(pos) => format!("name#{}", pos),
                LeaderBoardProperty::HotRound(pos) => format!("hrp{}", pos),
                LeaderBoardProperty::RoundScore(pos) => format!("rs#{}", pos),
                LeaderBoardProperty::TotalScore { pos, .. } => {
                    format!("ts#{}", pos)
                }
                LeaderBoardProperty::TotalScoreTitle => "ts".to_string(),
                LeaderBoardProperty::Move { pos, .. } => {
                    format!("move{}", pos)
                }
                LeaderBoardProperty::Arrow { pos, .. } => {
                    format!("arw{}", pos)
                }
                LeaderBoardProperty::Thru(pos) => format!("thru#{}", pos),
                LeaderBoardProperty::CheckinText => "checkintext".to_string(),
            }
        }
        fn data_extension(&self) -> &'static str {
            match self {
                LeaderBoardProperty::HotRound(_) | LeaderBoardProperty::Arrow { .. } => "Source",
                _ => "Text",
            }
        }

        fn value(&self) -> Option<String> {
            match self {
                LeaderBoardProperty::Position { tied, lb_pos, .. } => {
                    if tied.is_some() {
                        Some(format!("T{}", lb_pos))
                    } else {
                        Some(lb_pos.to_string())
                    }
                }

                _ => None,
            }
        }
        const INPUT_ID: &'static str = "38ded319-d270-41ec-b161-130db4b19901";
    }

}
use crate::flipup_vmix_controls::Image;
pub use prop::LeaderBoardProperty;
use crate::controller::queries::Division;

#[cfg(test)]
mod test {
    use super::{Leaderboard, LeaderboardState};
    use crate::controller::Player;
    use std::collections::HashSet;

    use crate::controller::get_data::{HoleResult, PlayerRound};
    use crate::controller::queries::layout::hole::Hole;
    use crate::controller::queries::layout::Holes;
    use crate::vmix::Queue;
    use fake::faker::name::en::{FirstName, LastName};
    use fake::utils::AlwaysTrueRng;
    use fake::uuid::UUIDv4;
    use fake::{Dummy, Fake, Faker};
    use itertools::Itertools;
    use rand::rngs::{OsRng, StdRng};
    use rand::SeedableRng;

    #[derive(Debug, Dummy)]
    struct TestingPlayer {
        user: TestingUser,
        #[dummy(faker = "UUIDv4")]
        id: String,
        #[dummy(faker = "(Faker, 18)")]
        results: Vec<TestingResult>,
    }
    #[derive(Debug, Dummy)]
    struct TestingUser {
        #[dummy(faker = "FirstName()")]
        pub first_name: String,
        #[dummy(faker = "LastName()")]
        pub last_name: String,
    }

    #[derive(Debug, Dummy)]
    struct TestingResult {
        #[dummy(faker = "1..=8")]
        pub throws: u8,
    }

    #[derive(Debug, Dummy)]
    struct TestingHoles {
        #[dummy(faker = "(Faker, 18)")]
        holes: Vec<TestingHole>,
    }
    #[derive(Debug, Dummy)]
    struct TestingHole {
        #[dummy(faker = "500..2000")]
        length: u16,
        #[dummy(faker = "3..6")]
        par: u8,
    }

    impl From<TestingHoles> for Holes {
        fn from(value: TestingHoles) -> Self {
            let holes = value
                .holes
                .into_iter()
                .enumerate()
                .map(|(i, hole)| Hole::from_testing(hole.par, hole.length, (i + 1) as u8))
                .collect_vec();
            Holes::from(holes)
        }
    }

    impl Hole {
        fn from_testing(par: u8, length: u16, number: u8) -> Self {
            Self {
                length,
                par,
                hole: number,
            }
        }
    }
    impl Holes {
        fn dummy() -> Self {
            let holes: TestingHoles = Faker.fake();
            holes.into()
        }
    }
    impl From<TestingUser> for crate::controller::queries::User {
        fn from(value: TestingUser) -> Self {
            Self {
                first_name: Some(value.first_name),
                last_name: Some(value.last_name),
                profile: None,
            }
        }
    }

    impl Player {
        fn new_test(player: TestingPlayer, holes: Holes) -> Self {
            let results = player
                .results
                .into_iter()
                .enumerate()
                .map(|(i, result)| {
                    let i = i + 1;
                    HoleResult {
                        hole: i as u8,
                        throws: result.throws,
                        hole_representation: holes.find_hole(i as u8).unwrap(),
                        tjing_result: None,
                        ob: HashSet::new(),
                        finished: true,
                    }
                })
                .collect_vec();
            let round = PlayerRound::new(results, 1);
            let first_name = player.user.first_name;
            let surname = player.user.last_name;
            Self {
                player_id: player.id,
                results: round,
                first_name: first_name.clone(),
                surname: surname.clone(),
                name: format!("{} {}", first_name, surname),
                round_ind: 1,
                ..Default::default()
            }
        }
    }

    fn make_many_players(holes: Holes) -> Vec<Player> {
        (0..10)
            .map(|_| {
                let fake: TestingPlayer = Faker.fake();
                Player::new_test(fake, holes.clone())
            })
            .collect_vec()
    }

    #[tokio::test]
    async fn test() {
        let holes = Holes::dummy();
        let p = LeaderboardState::new(1, make_many_players(holes));
        let a = Leaderboard::new(p.clone());
        let funcs = a.to_vmix_instructions();
        let q = Queue::new("10.170.120.134".to_string()).unwrap();
        q.add(&funcs);
        tokio::time::sleep(tokio::time::Duration::new(1, 0)).await;
    }
}
