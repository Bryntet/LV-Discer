use std::sync::Arc;

use itertools::Itertools;
use rayon::prelude::*;
use rocket::http::ext::IntoCollection;

use crate::controller::coordinator::BroadcastType;
use crate::controller::fix_score;
use crate::controller::get_data::HoleResult;
use crate::controller::queries::Division;
use crate::controller::Player;
use crate::flipup_vmix_controls::leaderboard::prop::FeaturedLeaderboard;
use crate::flipup_vmix_controls::Image;
use crate::vmix::functions::{VMixInterfacer, VMixSelectionTrait};
use crate::vmix::VMixQueue;
pub use prop::{CycledLeaderboard, LeaderBoardProperty, LeaderboardTop6};

#[derive(Debug, Clone, Default)]
pub struct Leaderboard {
    states: Vec<LeaderboardState>,
    pub skip: usize,
    pub cycle: bool,
    broadcast_type: Arc<BroadcastType>,
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
    pub fn all_players_in_div(
        &self,
        division: Arc<Division>,
        round: usize,
    ) -> Vec<LeaderboardPlayer> {
        self.current_state(round)
            .unwrap()
            .leaderboard_players(&division, self.previous_state(round))
    }
    fn current_state(&self, round: usize) -> Option<&LeaderboardState> {
        let state = match self.broadcast_type.as_ref() {
            BroadcastType::PostLive => self.find_state_by_round(round),
            BroadcastType::Live => self.find_state_by_round(round),
        };

        if state.is_none() {
            self.states.first()
        } else {
            state
        }
    }

    /// Returns the previous state of the leaderboard if it exists
    fn previous_state(&self, round: usize) -> Option<&LeaderboardState> {
        if self.states.len() <= 1 || round < 1 {
            return None;
        }

        match self.broadcast_type.as_ref() {
            BroadcastType::PostLive => self.states.get(round.checked_sub(2)?),
            BroadcastType::Live => self.states.get(round.checked_sub(1)?),
        }
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

    pub fn send_to_vmix(
        &self,
        division: &Division,
        queue: Arc<VMixQueue>,
        round: usize,
        featured: bool,
    ) {
        self.current_state(round)
            .map(|state| {
                state.send_to_vmix(
                    self.cycle,
                    division,
                    self.previous_state(round),
                    queue.clone(),
                    self.skip,
                    featured,
                )
            })
            .expect("Should work")
    }

    pub fn add_state(&mut self, state: LeaderboardState) {
        if self
            .current_state(self.states.len().checked_sub(1).unwrap_or_default())
            .is_some_and(|current_state| current_state.round == state.round)
        {
            self.states.pop();
        }
        self.states.push(state)
    }

    pub fn get_lb_player(&self, player: &Player) -> Option<LeaderboardPlayer> {
        self.current_state(self.states.len() - 1)
            .unwrap()
            .leaderboard_players(&player.division, self.previous_state(self.states.len() - 1))
            .into_iter()
            .find(|lb_player| lb_player.id == player.player_id)
    }

    pub fn find_player_in_current_state(&self, player: &Player) -> Option<&Player> {
        self.find_state_by_round(player.round_ind)?
            .players
            .iter()
            .find(|p| p.player_id == player.player_id)
    }

    fn find_state_by_round(&self, round: usize) -> Option<&LeaderboardState> {
        self.states.iter().find(|state| state.round == round)
    }

    pub fn update_little_lb(&self, div: &Division, queue: Arc<VMixQueue>) {
        if let Some(current) = self.current_state(self.states.len() - 1) {
            let previous = match self.states.len().checked_sub(2) {
                Some(round) => self.previous_state(round),
                None => None,
            };
            let previous_batch = current.big_leaderboard_funcs(div, previous, 0);
            current.update_little_leaderboard::<CycledLeaderboard>(
                div,
                previous_batch,
                previous,
                queue,
                self.cycle,
                false,
            );
        }
    }

    pub fn clear_little_lb(&self, queue: Arc<VMixQueue>) {}
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
            match (player_a.dnf || player_a.dns, player_b.dnf || player_b.dns) {
                (true, true) => std::cmp::Ordering::Equal,
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                (false, false) => {
                    let cmp = player_a.total_score.cmp(&player_b.total_score);
                    match cmp {
                        std::cmp::Ordering::Equal => {
                            let cmp = player_a.round_score.cmp(&player_b.round_score);
                            match cmp {
                                std::cmp::Ordering::Equal => {
                                    player_a.pdga_num.cmp(&player_b.pdga_num)
                                }
                                _ => cmp,
                            }
                        }
                        _ => cmp,
                    }
                }
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
                .filter(|player| player.division.name == division.name)
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
        cycled: bool,
        division: &Division,
        other: Option<&Self>,
        queue: Arc<VMixQueue>,
        skip: usize,
        featured: bool,
    ) {
        let first_batch = self.big_leaderboard_funcs(division, other, skip);

        queue.add_ref(first_batch.iter());
        let func = if featured {
            Self::update_little_leaderboard::<FeaturedLeaderboard>
        } else {
            Self::update_little_leaderboard::<CycledLeaderboard>
        };
        if skip > 0 {
            func(
                self,
                division,
                self.big_leaderboard_funcs(division, other, 0),
                other,
                queue,
                cycled,
                featured,
            )
        } else {
            func(self, division, first_batch, other, queue, cycled, featured);
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
            format!("{} | Round {}", division.short_name, self.round + 1),
            LeaderBoardProperty::CheckinText,
        ));
        funcs
    }

    pub fn update_little_leaderboard<S>(
        &self,
        division: &Division,
        first_batch: Vec<VMixInterfacer<LeaderBoardProperty>>,
        other: Option<&Self>,
        queue: Arc<VMixQueue>,
        cycled: bool,
        featured: bool,
    ) where
        S: VMixSelectionTrait,
        VMixInterfacer<S>: From<VMixInterfacer<LeaderboardTop6>>,
    {
        let lb_players = self.leaderboard_players(division, other);

        let mut second_batch: Vec<_> = first_batch
            .into_iter()
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
            .flatten();

        second_batch.extend(v);

        info!("Update little lb length: {}", second_batch.len());
        if cycled || featured {
            queue.add(second_batch.into_iter().map(VMixInterfacer::<S>::from));
        } else {
            queue.add_ref(second_batch.iter());
        }
    }
}

#[derive(Debug, Clone, Default)]
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
    dns: bool,
    dnf: bool,
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
            dns: player.dns,
            dnf: player.dnf,
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
            LeaderBoardProperty::HotRound(self.index),
        )
    }

    fn set_round_score(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            fix_score(self.round_score),
            LeaderBoardProperty::RoundScore(self.index),
        )
    }

    fn set_total_score(&self) -> VMixInterfacer<LeaderBoardProperty> {
        VMixInterfacer::set_text(
            fix_score(self.total_score),
            LeaderBoardProperty::TotalScore { pos: self.index },
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
            } else if self.dns {
                "DNS".to_string()
            } else if self.dnf {
                "DNF".to_string()
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

#[derive(Debug, Clone, Default)]
pub enum LeaderboardMovement {
    Up(usize),
    Down(usize),
    #[default]
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
    use crate::vmix::functions::{VMixInterfacer, VMixSelectionTrait};

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
                    format!("pos#{pos}")
                }
                LeaderBoardProperty::Name(pos) => format!("name#{pos}"),
                LeaderBoardProperty::HotRound(pos) => format!("hrp{pos}"),
                LeaderBoardProperty::RoundScore(pos) => format!("rs#{pos}"),
                LeaderBoardProperty::TotalScore { pos, .. } => {
                    format!("ts#{pos}")
                }
                LeaderBoardProperty::TotalScoreTitle => "ts".to_string(),
                LeaderBoardProperty::Move { pos, .. } => {
                    format!("move{pos}")
                }
                LeaderBoardProperty::Arrow { pos, .. } => {
                    format!("arw{pos}")
                }
                LeaderBoardProperty::Thru(pos) => format!("thru#{pos}"),
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
        fn input_id(&self) -> &'static str {
            "38ded319-d270-41ec-b161-130db4b19901"
        }
    }

    pub struct FeaturedLeaderboard(LeaderboardTop6);

    impl VMixSelectionTrait for FeaturedLeaderboard {
        fn get_selection_name(&self) -> String {
            self.0.get_selection_name()
        }

        fn data_extension(&self) -> &'static str {
            self.0.data_extension()
        }

        fn value(&self) -> Option<String> {
            self.0.value()
        }

        fn input_id(&self) -> &'static str {
            "1900db1a-4f83-4111-848d-d9a87474f56c"
        }
    }
    impl From<VMixInterfacer<LeaderboardTop6>> for VMixInterfacer<FeaturedLeaderboard> {
        fn from(interfacer: VMixInterfacer<LeaderboardTop6>) -> Self {
            let value = interfacer.value;
            let function = interfacer.function;

            Self {
                value,
                function,
                input: interfacer.input.map(FeaturedLeaderboard),
            }
        }
    }
    pub struct CycledLeaderboard(LeaderboardTop6);

    impl VMixSelectionTrait for CycledLeaderboard {
        fn get_selection_name(&self) -> String {
            self.0.get_selection_name()
        }

        fn data_extension(&self) -> &'static str {
            self.0.data_extension()
        }

        fn value(&self) -> Option<String> {
            self.0.value()
        }

        fn input_id(&self) -> &'static str {
            // TODO SET ID
            "712e8a3d-605f-4ce2-b8c5-74f64f903495"
        }
    }

    impl From<VMixInterfacer<LeaderboardTop6>> for VMixInterfacer<CycledLeaderboard> {
        fn from(interfacer: VMixInterfacer<LeaderboardTop6>) -> Self {
            let value = interfacer.value;
            let function = interfacer.function;

            Self {
                value,
                function,
                input: interfacer.input.map(CycledLeaderboard),
            }
        }
    }
    pub enum LeaderboardTop6 {
        Position { pos: usize },
        Name { pos: usize },
        LastScore { pos: usize, hole: usize },
        LastScoreColour { pos: usize, hole: usize },
        RoundScore { pos: usize },
        TotalScore { pos: usize },
        Thru { pos: usize },
        DivisionName,
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
                DivisionName => "top6txt".to_string(),
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

        fn input_id(&self) -> &'static str {
            "1900db1a-4f83-4111-848d-d9a87474f56c"
        }
    }

    impl LeaderboardTop6 {
        pub fn from_prop(leader_board_property: LeaderBoardProperty) -> Option<Self> {
            match leader_board_property {
                LeaderBoardProperty::TotalScoreTitle
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
                LeaderBoardProperty::CheckinText => Some(LeaderboardTop6::DivisionName),
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
