use itertools::Itertools;

use crate::controller::hole::{DroneHoleInfo, FeaturedHole, VMixHoleInfo};
use crate::flipup_vmix_controls::{LeaderBoardProperty, LeaderboardTop6};

pub trait VMixSelectionTrait {
    fn get_selection(&self) -> String {
        let name = self.get_selection_name();
        let extension = self.data_extension();

        if let Some(value) = self
            .value()
            .and_then(|val| if val.is_empty() { None } else { Some(val) })
        {
            format!(
                "Input={}&SelectedName={name}.{extension}&Value={value}",
                self.input_id()
            )
        } else {
            format!("Input={}&SelectedName={name}.{extension}", self.input_id())
        }
    }
    fn get_selection_name(&self) -> String;

    fn data_extension(&self) -> &'static str;

    fn value(&self) -> Option<String>;

    fn input_id(&self) -> &'static str;
}

#[derive(Clone)]
pub struct VMixInterfacer<InputEnum: VMixSelectionTrait> {
    pub value: Option<String>,
    pub input: Option<InputEnum>,
    pub function: VMixFunction,
}

impl VMixInterfacer<LeaderBoardProperty> {
    pub fn to_top_6(self) -> Option<VMixInterfacer<LeaderboardTop6>> {
        let input = LeaderboardTop6::from_prop(self.input.clone()?)?;
        match self.input {
            Some(LeaderBoardProperty::Name(n)) => {
                let name = self.value.unwrap();
                let name = name.split(" ").collect_vec();
                let name = format!("{}. {}", name[0].chars().next().unwrap(), name[1]);
                Some(VMixInterfacer {
                    value: Some(name),
                    function: self.function,
                    input: Some(LeaderboardTop6::Name { pos: n }),
                })
            }
            _ => Some(VMixInterfacer {
                value: self.value,
                function: self.function,
                input: Some(input),
            }),
        }
    }
}

impl<T: VMixSelectionTrait> VMixInterfacer<T> {
    pub fn set_only_input(input: T) -> Self {
        Self {
            value: None,
            input: Some(input),
            function: VMixFunction::SetText,
        }
    }
}

// Functions initialisers
impl<InputEnum: VMixSelectionTrait> VMixInterfacer<InputEnum> {
    pub fn set_text(value: String, input: InputEnum) -> Self {
        Self {
            value: Some(value),
            input: Some(input),
            function: VMixFunction::SetText,
        }
    }

    pub fn set_color(value: &str, input: InputEnum) -> Self {
        Self {
            value: Some(format!("#{}", value)),
            input: Some(input),
            function: VMixFunction::SetColor,
        }
    }

    pub fn set_text_visible_on(input: InputEnum) -> Self {
        Self {
            value: None,
            input: Some(input),
            function: VMixFunction::SetTextVisibleOn,
        }
    }
    pub fn set_text_visible_off(input: InputEnum) -> Self {
        Self {
            value: None,
            input: Some(input),
            function: VMixFunction::SetTextVisibleOff,
        }
    }

    pub fn set_image(value: String, input: InputEnum) -> Self {
        Self {
            value: Some(value),
            input: Some(input),
            function: VMixFunction::SetImage,
        }
    }

    pub fn overlay_input_4_off() -> Self {
        Self {
            value: None,
            input: None,
            function: VMixFunction::OverlayInput4Off,
        }
    }

    pub fn overlay_input_4(value: &'static str) -> Self {
        Self {
            value: Some(value.to_string()),
            input: None,
            function: VMixFunction::OverlayInput4,
        }
    }
}

#[derive(Clone, Debug)]
pub enum VMixFunction {
    SetText,
    SetColor,
    SetTextVisibleOn,
    SetTextVisibleOff,
    SetImage,
    OverlayInput4Off,
    OverlayInput4,
}

impl<InputEnum: VMixSelectionTrait> VMixInterfacer<InputEnum> {
    fn get_input(&self) -> Option<String> {
        match self.function {
            VMixFunction::OverlayInput4 => Some(format!("Input={}", self.value.as_ref().unwrap())),
            VMixFunction::OverlayInput4Off => None,
            _ => self.input.as_ref().map(|i| i.get_selection().to_owned()),
        }
    }

    fn get_value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    pub fn to_cmd(&self) -> String {
        let cmd = self.function.get_start_cmd();
        let input = self.get_input();
        let value = self.get_value();

        "FUNCTION ".to_string()
            + &match (input, value) {
                (Some(input), Some(value)) => format!("{cmd} {input}&Value={value}",),
                (Some(input), None) => format!("{cmd} {input}"),
                (None, Some(value)) => format!("{cmd} Value={value}"),
                (None, None) => cmd.to_string(),
            }
            + "\r\n"
    }
}
impl VMixFunction {
    const fn get_start_cmd(&self) -> &'static str {
        match self {
            VMixFunction::SetText => "SetText",
            VMixFunction::SetColor => "SetColor",
            VMixFunction::SetTextVisibleOn => "SetTextVisibleOn",
            VMixFunction::SetTextVisibleOff => "SetTextVisibleOff",
            VMixFunction::SetImage => "SetImage",
            VMixFunction::OverlayInput4Off => "OverlayInput4Off",
            VMixFunction::OverlayInput4 => "OverlayInput4",
        }
    }
}

#[derive(Clone, Debug)]
pub enum VMixPlayerInfo {
    Score { hole: usize, player: usize },
    ScoreColor { hole: usize, player: usize },
    Name(usize),
    Surname(usize),
    TotalScore(usize),
    RoundScore(usize),
    Throw(usize),
    PlayerPosition(usize),
    PositionArrow(usize),
    PositionMove(usize),
    HotRound(usize),
    InsidePutt(usize),
    CircleHit(usize),
}

impl VMixSelectionTrait for VMixPlayerInfo {
    fn get_selection_name(&self) -> String {
        match self {
            VMixPlayerInfo::Score { hole, player } => {
                format!("p{}s{}", player + 1, hole)
            }

            VMixPlayerInfo::ScoreColor { hole, player } => {
                format!("p{}h{}", player + 1, hole)
            }
            VMixPlayerInfo::PositionArrow(n) => {
                format!("p{}posarw", n + 1)
            }
            VMixPlayerInfo::PositionMove(n) => {
                format!("p{}posmove", n + 1)
            }
            VMixPlayerInfo::Name(ind) => format!("p{}name", ind + 1),
            VMixPlayerInfo::Surname(ind) => format!("p{}surname", ind + 1),
            VMixPlayerInfo::TotalScore(ind) => format!("p{}scoretot", ind + 1),
            VMixPlayerInfo::RoundScore(ind) => format!("p{}scorernd", ind + 1),
            VMixPlayerInfo::Throw(ind) => format!("p{}throw", ind + 1),
            VMixPlayerInfo::PlayerPosition(pos) => format!("p{}pos", pos + 1),
            VMixPlayerInfo::HotRound(pos) => format!("p{}hotrnd", pos + 1),
            VMixPlayerInfo::CircleHit(pos) => format!("p{}c1reg", pos + 1),
            VMixPlayerInfo::InsidePutt(pos) => format!("p{}c1x", pos + 1),
        }
    }

    fn data_extension(&self) -> &'static str {
        use VMixPlayerInfo::*;
        match self {
            Name(_)
            | Throw(_)
            | PlayerPosition(_)
            | RoundScore(_)
            | Surname(_)
            | Score { .. }
            | TotalScore(_)
            | PositionMove(_)
            | InsidePutt(_)
            | CircleHit(_) => "Text",
            ScoreColor { .. } => "Fill.Color",
            PositionArrow(_) | HotRound(_) => "Source",
        }
    }
    fn value(&self) -> Option<String> {
        None
    }

    fn input_id(&self) -> &'static str {
        "8db7c455-e05c-4e65-821b-048cd7057cb1"
    }
}

pub struct CurrentPlayer(pub VMixPlayerInfo);

impl VMixSelectionTrait for CurrentPlayer {
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
        "03a31701-8b74-46f9-a9e0-9e263d7ba0be"
    }
}

impl VMixInterfacer<VMixPlayerInfo> {
    pub fn into_current_player(self) -> Option<VMixInterfacer<CurrentPlayer>> {
        let input = match self.input {
            Some(VMixPlayerInfo::Score { .. } | VMixPlayerInfo::ScoreColor { .. }) => None,
            Some(i) => Some(i),
            None => None,
        }?;

        Some(VMixInterfacer {
            input: Some(CurrentPlayer(input)),
            function: self.function,
            value: self.value,
        })
    }

    pub fn into_compare_2x2_player(mut self, index: usize) -> VMixInterfacer<Compare2x2> {
        if let Some(input) = &mut self.input {
            match input {
                VMixPlayerInfo::Score { player, .. }
                | VMixPlayerInfo::ScoreColor { player, .. } => {
                    *player = index;
                }

                VMixPlayerInfo::Name(n)
                | VMixPlayerInfo::Surname(n)
                | VMixPlayerInfo::TotalScore(n)
                | VMixPlayerInfo::RoundScore(n)
                | VMixPlayerInfo::Throw(n)
                | VMixPlayerInfo::PlayerPosition(n)
                | VMixPlayerInfo::PositionArrow(n)
                | VMixPlayerInfo::PositionMove(n)
                | VMixPlayerInfo::HotRound(n)
                | VMixPlayerInfo::InsidePutt(n)
                | VMixPlayerInfo::CircleHit(n) => {
                    *n = index;
                }
            }
        }

        VMixInterfacer {
            input: self.input.map(Compare2x2::Standard),
            function: self.function,
            value: self.value,
        }
    }
}

impl VMixInterfacer<VMixHoleInfo> {
    pub fn into_drone_hole_info(self) -> VMixInterfacer<DroneHoleInfo> {
        VMixInterfacer {
            input: self.input.map(DroneHoleInfo::Standard),
            function: self.function,
            value: self.value,
        }
    }

    pub fn into_featured_hole_card(self) -> VMixInterfacer<FeaturedHole> {
        VMixInterfacer {
            input: self.input.map(FeaturedHole),
            function: self.function,
            value: self.value,
        }
    }
}

pub enum Compare2x2 {
    Standard(VMixPlayerInfo),
    PlayerImage { index: usize },
}

impl VMixSelectionTrait for Compare2x2 {
    fn get_selection_name(&self) -> String {
        match self {
            Compare2x2::Standard(s) => s.get_selection_name(),
            Compare2x2::PlayerImage { index } => format!("pimg{}", index + 1),
        }
    }
    fn data_extension(&self) -> &'static str {
        match self {
            Compare2x2::Standard(s) => s.data_extension(),
            Compare2x2::PlayerImage { .. } => "Source",
        }
    }
    fn value(&self) -> Option<String> {
        match self {
            Compare2x2::Standard(s) => s.value(),
            Compare2x2::PlayerImage { .. } => None,
        }
    }

    fn input_id(&self) -> &'static str {
        "a4f106c7-db2c-4aa8-895b-076ba55de8a7"
    }
}

pub struct Featured(Compare2x2);

impl VMixInterfacer<Compare2x2> {
    pub fn into_featured(self) -> VMixInterfacer<Featured> {
        VMixInterfacer {
            function: self.function,
            value: self.value,
            input: self.input.map(Featured),
        }
    }
}

impl VMixSelectionTrait for Featured {
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
        "2994000d-afe5-44fc-a2c1-fc0993de21da"
    }
}
