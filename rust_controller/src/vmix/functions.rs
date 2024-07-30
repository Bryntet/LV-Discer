use crate::flipup_vmix_controls::{LeaderBoardProperty, LeaderboardTop6};
use crate::vmix::functions::VMixFunction::OverlayInput4Off;
use std::sync::Arc;

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
                Self::INPUT_ID
            )
        } else {
            format!("Input={}&SelectedName={name}.{extension}", Self::INPUT_ID)
        }
    }
    fn get_selection_name(&self) -> String;

    fn data_extension(&self) -> &'static str;

    fn value(&self) -> Option<String>;

    const INPUT_ID: &'static str;
}

pub struct VMixInterfacer<InputEnum: VMixSelectionTrait> {
    value: Option<String>,
    input: Option<InputEnum>,
    function: VMixFunction,
}

impl VMixInterfacer<LeaderBoardProperty> {
    pub fn to_top_6(self) -> Option<VMixInterfacer<LeaderboardTop6>> {
        let input = LeaderboardTop6::from_prop(self.input?)?;

        Some(VMixInterfacer {
            value: self.value,
            function: self.function,
            input: Some(input),
        })
    }
}

impl VMixInterfacer<VMixHoleInfo> {
    pub fn set_hole_text(input: VMixHoleInfo) -> Self {
        Self {
            value: None,
            input: Some(input),
            function: VMixFunction::SetText
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
    PosRightTriColor(usize),
    PosSquareColor(usize),
    Name(usize),
    Surname(usize),
    TotalScore(usize),
    RoundScore(usize),
    Throw(usize),
    PlayerPosition(u16),
}

impl VMixSelectionTrait for VMixPlayerInfo {
    fn get_selection_name(&self) -> String {
        match self {
            VMixPlayerInfo::Score { hole, .. } => {
                format!("p{}s{}", 1, hole)
            }

            VMixPlayerInfo::ScoreColor { hole, .. } => {
                format!("p{}h{}", 1, hole)
            }
            VMixPlayerInfo::PosRightTriColor(v1) => {
                format!("RightTriangle{}", v1 + 1)
            }
            VMixPlayerInfo::PosSquareColor(v1) => {
                format!("Rectangle{}", v1 + 1)
            }
            VMixPlayerInfo::Name(ind) => format!("p{}name", ind + 1),
            VMixPlayerInfo::Surname(ind) => format!("p{}surname", ind + 1),
            VMixPlayerInfo::TotalScore(ind) => format!("p{}scoretot", ind + 1),
            VMixPlayerInfo::RoundScore(ind) => format!("p{}scorernd", ind + 1),
            VMixPlayerInfo::Throw(ind) => format!("p{}throw", ind + 1),
            VMixPlayerInfo::PlayerPosition(pos) => format!("p{}pos", pos + 1),
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
            | TotalScore(_) => "Text",
            PosSquareColor(_) | PosRightTriColor(_) | ScoreColor { .. } => "Fill.Color",
        }
    }
    fn value(&self) -> Option<String> {
        None
    }

    const INPUT_ID: &'static str = "8db7c455-e05c-4e65-821b-048cd7057cb1";
}

#[derive(Clone, Debug)]
pub enum VMixHoleInfo {
    Hole(u8),
    HolePar(u8),
    HoleMeters(u16),
    HoleFeet(u16),
    AverageResult(f64),
    Difficulty {
        hole: usize,
        difficulty: Arc<HoleDifficulty>,
    },
    Elevation(i16),
}
#[derive(Clone, Debug)]
pub struct HoleDifficulty {
    holes: [usize; 18],
}

impl HoleDifficulty {
    fn hole_difficulty_text(&self, hole: usize) -> Option<String> {
        let difficulty = self.holes.get(hole)?;
        Some(match difficulty {
            1 => "EASIEST".to_string(),
            2 => "2nd easiest".to_string(),
            3 => "3d easiest".to_string(),
            4..=9 => format!("{difficulty}th easiest"),
            10..=15 => format!("{}th hardest", difficulty - 9),
            16 => "3d hardest".to_string(),
            17 => "2nd hardest".to_string(),
            18 => "HARDEST".to_string(),
            _ => None?,
        })
    }
}

impl VMixSelectionTrait for VMixHoleInfo {
    fn get_selection_name(&self) -> String {
        use VMixHoleInfo::*;
        match self {
            Hole(_) => "hole",
            HolePar(_) => "parnr",
            HoleMeters(_) => "meternr",
            HoleFeet(_) => "feetnr",
            AverageResult(_) => "avgresult",
            Difficulty { .. } => "difficulty",
            Elevation(_) => "elevation",
        }
        .to_string()
    }

    fn data_extension(&self) -> &'static str {
        "Text"
    }

    fn value(&self) -> Option<String> {
        use VMixHoleInfo::*;

        Some(match self {
            Hole(number) | HolePar(number) => number.to_string(),
            HoleMeters(meters) => format!("{meters}M"),
            HoleFeet(feet) => format!("{feet}FT"),
            AverageResult(number) => number.to_string(),
            Difficulty { difficulty, hole } => difficulty.hole_difficulty_text(*hole).unwrap(),
            Elevation(elevation) => {
                let sign = if elevation.is_positive() { "+" } else { "" };
                format!("{sign}{elevation}")
            }
        })
    }
    const INPUT_ID: &'static str = "d9806a48-8766-40e0-b7fe-b217f9b1ef5b";
}
