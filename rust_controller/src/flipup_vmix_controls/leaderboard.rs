use std::sync::Arc;

use itertools::Itertools;
use rayon::prelude::*;
use rocket::http::ext::IntoCollection;

pub use prop::{LeaderBoardProperty, LeaderboardTop6};

use crate::controller::fix_score;
use crate::controller::get_data::HoleResult;
use crate::controller::queries::Division;
use crate::controller::Player;
use crate::flipup_vmix_controls::Image;
use crate::vmix::functions::{VMixInterfacer, VMixSelectionTrait};
use crate::vmix::VMixQueue;

#[derive(Debug, Clone, Default)]
pub struct Leaderboard {
    states: Vec<LeaderboardState>,
    pub skip: usize,
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
            skip: 0,
        }
    }
    fn current_state(&self) -> Option<&LeaderboardState> {
        self.states.last()
    }

    /// Returns the previous state of the leaderboard if it exists
    fn previous_state(&self) -> Option<&LeaderboardState> {
        if self.states.len() <= 1 {
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

    pub fn send_to_vmix(&self, division: &Division, queue: Arc<VMixQueue>) {
        self.current_state()
            .map(|state| {
                state.send_to_vmix(division, self.previous_state(), queue.clone(), self.skip)
            })
            .expect("Should work")
    }

    pub fn add_state(&mut self, state: LeaderboardState) {
        if self
            .current_state()
            .is_some_and(|current_state| current_state.round == state.round)
        {
            self.states.pop();
        }
        self.states.push(state)
    }

    pub fn get_lb_player(&self, player: &Player) -> Option<LeaderboardPlayer> {
        self.current_state()
            .unwrap()
            .leaderboard_players(&player.division, self.previous_state())
            .into_iter()
            .find(|lb_player| lb_player.id == player.player_id)
    }

    pub fn find_player_in_current_state(&self, player: &Player) -> &Player {
        self.find_state_by_round(player.round_ind)
            .unwrap()
            .players
            .iter()
            .find(|p| p.player_id == player.player_id)
            .unwrap()
    }

    fn find_state_by_round(&self, round: usize) -> Option<&LeaderboardState> {
        self.states.iter().find(|state| state.round == round)
    }

    pub fn update_little_lb(&self, div: &Division, queue: Arc<VMixQueue>) {
        if let Some(current) = self.current_state() {
            let previous = self.previous_state();
            let previous_batch = current.big_leaderboard_funcs(div, previous, 0);
            current.update_little_leaderboard(div, previous_batch, previous, queue);
        }
    }
}
impl LeaderboardState {
    pub fn new(
        round: usize,
        mut current_round_players: Vec<Player>,
        mut all_previous_rounds_players: Vec<Player>,
    ) -> Self {
        current_round_players.iter_mut().for_each(|player| {
            let previous_instances = all_previous_rounds_players
                .iter_mut()
                .filter(|previous_round_player| previous_round_player.player_id == player.player_id)
                .collect_vec();
            let total = previous_instances
                .into_iter()
                .map(|player| {
                    player.fix_round_score(None);
                    player.round_score
                })
                .sum::<isize>();
            player.fix_round_score(None);
            player.total_score = total + player.round_score;
        });
        Self::sort_players(&mut current_round_players);
        Self {
            where_to_start: LeaderboardStart::Latest,
            round,
            players: current_round_players,
        }
    }

    fn sort_players(players: &mut [Player]) {
        players.sort_by(|player_a, player_b| {
            let cmp = player_a.total_score.cmp(&player_b.total_score);
            match cmp {
                std::cmp::Ordering::Equal => {
                    let cmp = player_a.name.cmp(&player_b.name);
                    match cmp {
                        std::cmp::Ordering::Equal => player_a.pdga_num.cmp(&player_b.pdga_num),
                        _ => cmp,
                    }
                }
                _ => cmp,
            }
        })
    }

    fn leaderboard_players(
        &self,
        division: &Division,
        other: Option<&Self>,
    ) -> Vec<LeaderboardPlayer> {
        let min_score = self
            .players
            .iter()
            .map(|p| p.round_score)
            .min()
            .unwrap_or_default();
        let other = other.map(|state| state.leaderboard_players(division, None));
        let players_with_pos = Self::players_with_positions(
            self.players
                .iter()
                .filter(|player| player.division.id == division.id)
                .collect_vec(),
        );

        players_with_pos
            .into_iter()
            .enumerate()
            .map(|(real_pos, (index, player))| {
                LeaderboardPlayer::new(
                    player,
                    index,
                    real_pos + 1,
                    min_score,
                    other.as_ref(),
                    self.round,
                    &self.players,
                )
            })
            .collect_vec()
    }

    fn players_with_positions(players: Vec<&Player>) -> Vec<(usize, &Player)> {
        let mut pos = 1;
        let mut same_score_count = 0;
        let mut last_score = players.first().map(|p| p.total_score).unwrap_or_default();

        players
            .into_iter()
            .map(|player| {
                if player.total_score != last_score {
                    pos += same_score_count;
                    same_score_count = 0;
                }
                same_score_count += 1;

                last_score = player.total_score;
                (pos, player)
            })
            .collect_vec()
    }

    pub fn send_to_vmix(
        &self,
        division: &Division,
        other: Option<&Self>,
        queue: Arc<VMixQueue>,
        skip: usize,
    ) {
        let first_batch = self.big_leaderboard_funcs(division, other, skip);
        queue.add(&first_batch);
        if skip > 0 {
            self.update_little_leaderboard(
                division,
                self.big_leaderboard_funcs(division, other, 0),
                other,
                queue,
            )
        } else {
            self.update_little_leaderboard(division, first_batch, other, queue);
        }
    }

    pub fn big_leaderboard_funcs(
        &self,
        division: &Division,
        other: Option<&Self>,
        skip: usize,
    ) -> Vec<VMixInterfacer<LeaderBoardProperty>> {
        let mut players = self.leaderboard_players(division, other);
        let mut funcs = players
            .iter_mut()
            .skip(skip * 10)
            .take(10)
            .map(|player| {
                player.index -= skip * 10;
                &*player
            })
            .flat_map(LeaderboardPlayer::combine)
            .collect_vec();
        funcs.push(VMixInterfacer::set_text(
            format!("{} | Round {} | Leaderboard", division.name, self.round + 1),
            LeaderBoardProperty::CheckinText,
        ));
        funcs
    }

    pub fn update_little_leaderboard(
        &self,
        division: &Division,
        first_batch: Vec<VMixInterfacer<LeaderBoardProperty>>,
        other: Option<&Self>,
        queue: Arc<VMixQueue>,
    ) {
        let lb_players = self.leaderboard_players(division, other);

        let mut second_batch: Vec<_> = first_batch
            .into_par_iter()
            .flat_map(VMixInterfacer::to_top_6)
            .collect();

        let v = lb_players
            .into_iter()
            .take(6)
            .flat_map(|player| {
                let regular_player = self
                    .players
                    .par_iter()
                    .find_any(|regular_player| regular_player.player_id == player.id)?;

                Some(
                    regular_player
                        .results
                        .clone()
                        .the_latest_6_holes(5)
                        .iter()
                        .enumerate()
                        .flat_map(|(hole_index, result)| match result {
                            Some(res) => res.to_leaderboard_top_6(player.index, hole_index + 1),
                            None => HoleResult::hide_hole_top_6(player.index, hole_index + 1),
                        })
                        .collect_vec(),
                )
            })
            .flatten()
            .collect_vec();

        second_batch.extend(v);
        queue.add(&second_batch);
    }
}

#[derive(Debug, Clone)]
pub struct LeaderboardPlayer {
    pub id: String,
    pub index: usize,
    pub position: usize,
    pub movement: LeaderboardMovement,
    pub hot_round: bool,
    name: String,
    pub round_score: isize,
    pub total_score: isize,
    thru: u8,
    pub tied: Option<u8>,
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
        min_score_reached: isize,
        other_board: Option<&Vec<LeaderboardPlayer>>,
        round: usize,
        all_other_players: &[Player],
    ) -> Self {
        let other_pos = other_board
            .and_then(|players| {
                players
                    .iter()
                    .find(|other_player| other_player.id == player.player_id)
            })
            .map(|player| player.position);
        let movement = match other_pos {
            Some(other_pos) => LeaderboardMovement::new(pos, other_pos),
            None => LeaderboardMovement::Same,
        };
        let tie = {
            let tie_count = all_other_players
                .iter()
                .filter(|lb_player| lb_player.division.id == player.division.id)
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
            hot_round: player.round_score == min_score_reached && round != 0,
            name: player.name.clone(),
            round_score: player.round_score,
            total_score: player.total_score,
            thru: player.results.amount_of_holes_finished(),
            tied: tie,
            id: player.player_id.clone(),
        }
    }

    fn set_hot_round(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_image(
            if self.hot_round {
                Image::Flames
            } else {
                Image::Nothing
            }
            .to_location(),
            LeaderBoardProperty::HotRound(self.index).into(),
        )
    }

    fn set_round_score(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            fix_score(self.round_score),
            LeaderBoardProperty::RoundScore(self.index).into(),
        )
    }

    fn set_total_score(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            fix_score(self.total_score),
            LeaderBoardProperty::TotalScore { pos: self.index }.into(),
        )
    }

    fn set_position(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            if self.tied.is_some() {
                format!("T{}", self.position)
            } else {
                self.position.to_string()
            },
            LeaderBoardProperty::Position { pos: self.index },
        )
    }

    fn set_movement_img(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_image(
            match self.movement {
                LeaderboardMovement::Up(_) => Image::RedTriDown,
                LeaderboardMovement::Down(_) => Image::GreenTriUp,
                LeaderboardMovement::Same => Image::Nothing,
            }
            .to_location(),
            LeaderBoardProperty::Arrow { pos: self.index },
        )
    }

    fn set_movement_text(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            match self.movement {
                LeaderboardMovement::Up(n) | LeaderboardMovement::Down(n) => n.to_string(),
                LeaderboardMovement::Same => " ".to_string(),
            },
            LeaderBoardProperty::Move { pos: self.index }.into(),
        )
    }

    fn set_thru(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            if self.thru == 18 {
                "F".to_string()
            } else {
                self.thru.to_string()
            },
            LeaderBoardProperty::Thru(self.index).into(),
        )
    }

    fn set_name(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            self.name.clone(),
            LeaderBoardProperty::Name(self.index).into(),
        )
    }

    pub fn combine(&self) -> Vec<VMixInterfacer<LeaderBoardProperty>> {
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

#[derive(Debug, Clone)]
pub enum LeaderboardMovement {
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
    use crate::vmix::functions::VMixSelectionTrait;

    #[derive(Clone)]
    pub enum LeaderBoardProperty {
        Position { pos: usize },
        Name(usize),
        HotRound(usize),
        RoundScore(usize),
        TotalScore { pos: usize },
        Move { pos: usize },
        Arrow { pos: usize },
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
            None
        }
        const INPUT_ID: &'static str = "38ded319-d270-41ec-b161-130db4b19901";
    }

    pub enum LeaderboardTop6 {
        Position { pos: usize },
        Name { pos: usize },
        LastScore { pos: usize, hole: usize },
        LastScoreColour { pos: usize, hole: usize },
        RoundScore { pos: usize },
        TotalScore { pos: usize },
        Thru { pos: usize },
    }
    impl VMixSelectionTrait for LeaderboardTop6 {
        fn get_selection_name(&self) -> String {
            use LeaderboardTop6::*;
            match self {
                Position { pos, .. } => format!("pos{pos}"),
                Name { pos } => format!("name{pos}"),
                LastScore { pos, hole } => format!("p{pos}ls{hole}",),
                LastScoreColour { pos, hole } => format!("p{pos}lh{hole}"),
                RoundScore { pos } => format!("p{pos}scorernd"),
                TotalScore { pos } => format!("p{pos}scoretot"),
                Thru { pos } => format!("p{pos}thru"),
            }
        }
        fn data_extension(&self) -> &'static str {
            match self {
                LeaderboardTop6::LastScoreColour { .. } => "Fill.Color",
                _ => "Text",
            }
        }
        fn value(&self) -> Option<String> {
            None
        }

        const INPUT_ID: &'static str = "1900db1a-4f83-4111-848d-d9a87474f56c";
    }

    impl LeaderboardTop6 {
        pub fn from_prop(leader_board_property: LeaderBoardProperty) -> Option<Self> {
            match leader_board_property {
                LeaderBoardProperty::CheckinText
                | LeaderBoardProperty::TotalScoreTitle
                | LeaderBoardProperty::Arrow { .. }
                | LeaderBoardProperty::HotRound(_)
                | LeaderBoardProperty::Move { .. } => None,
                LeaderBoardProperty::Position { pos } => Some(LeaderboardTop6::Position { pos }),
                LeaderBoardProperty::Thru(pos) => Some(LeaderboardTop6::Thru { pos }),
                LeaderBoardProperty::TotalScore { pos } => {
                    Some(LeaderboardTop6::TotalScore { pos })
                }
                LeaderBoardProperty::RoundScore(pos) => Some(LeaderboardTop6::RoundScore { pos }),

                LeaderBoardProperty::Name(pos) => Some(LeaderboardTop6::Name { pos }),
            }
        }
    }
}
#[cfg(test)]
mod test {
    use fake::faker::name::en::{FirstName, LastName};
    use fake::uuid::UUIDv4;
    use fake::{Dummy, Fake, Faker};

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

    /*impl From<TestingHoles> for Holes {
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
        let p = LeaderboardState::new(1, make_many_players(holes), vec![]);
        let a = Leaderboard::new(p.clone());
        let funcs = a.send_to_vmix();
        let q = VMixQueue::new("10.170.120.134".to_string()).unwrap();
        q.add(&funcs);
        tokio::time::sleep(tokio::time::Duration::new(1, 0)).await;
    }*/
}
