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
#[derive(Clone, Debug)]
pub struct VMixSelection<T: VMixSelectionTrait>(pub T);

impl<T: VMixSelectionTrait> VMixSelection<T> {
    fn get_selection(&self) -> String {
        self.0.get_selection()
    }
}

impl<T: VMixSelectionTrait> From<T> for VMixSelection<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}


#[derive(Clone, Debug)]
pub enum VMixFunction<InputEnum: VMixSelectionTrait> {
    SetText {
        value: String,
        input: VMixSelection<InputEnum>,
    },
    SetPanX {
        pan: f64,
    },
    SetColor {
        color: &'static str,
        input: VMixSelection<InputEnum>,
    },
    SetTextVisibleOn {
        input: VMixSelection<InputEnum>,
    },
    SetTextVisibleOff {
        input: VMixSelection<InputEnum>,
    },
    SetImage {
        value: String,
        input: VMixSelection<InputEnum>,
    },
    OverlayInput4Off,
    OverlayInput4(&'static str),
}

impl<InputEnum: VMixSelectionTrait> VMixFunction<InputEnum> {
    fn get_input(&self) -> Option<String> {
        use VMixFunction::*;
        match self {
            SetText { input, .. } => Some(input.get_selection()),
            SetColor { input, .. } => Some(input.get_selection()),
            OverlayInput4(mov) => Some(format!("Input={}", mov)),
            OverlayInput4Off | SetPanX { .. } => None,
            SetImage { input, .. } => Some(input.get_selection()),
            SetTextVisibleOn { input } => Some(input.get_selection()),
            SetTextVisibleOff { input } => Some(input.get_selection()),
        }
    }

    fn get_value(&self) -> Option<String> {
        match self {
            Self::SetText { value, .. } => {
                if !value.is_empty() {
                    Some(value.clone())
                } else {
                    None
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            Self::SetColor { color, .. } => Some("#".to_string() + color),
            #[cfg(target_arch = "wasm32")]
            Self::SetColor { color, .. } => Some("%23".to_string() + color),
            Self::OverlayInput4Off => None,
            Self::OverlayInput4(_) => None,
            Self::SetImage { value, .. } => Some(value.to_string()),
            Self::SetPanX { pan: value, .. } => Some(value.to_string()),
            Self::SetTextVisibleOn { .. } => None,
            Self::SetTextVisibleOff { .. } => None,
        }
        .map(|value| "Value=".to_string() + &value)
    }
}

impl<InputEnum: VMixSelectionTrait> VMixFunction<InputEnum> {
    const fn get_start_cmd(&self) -> &'static str {
        match self {
            VMixFunction::SetText { .. } => "SetText",
            VMixFunction::SetColor { .. } => "SetColor",
            VMixFunction::SetPanX { .. } => "SetPanX",
            VMixFunction::SetTextVisibleOn { .. } => "SetTextVisibleOn",
            VMixFunction::SetTextVisibleOff { .. } => "SetTextVisibleOff",
            VMixFunction::SetImage { .. } => "SetImage",
            VMixFunction::OverlayInput4Off => "OverlayInput4Off",
            VMixFunction::OverlayInput4(_) => "OverlayInput4",
        }
    }

    pub fn to_cmd(&self) -> String {
        let cmd = self.get_start_cmd();
        let input = self.get_input();
        let value = self.get_value();

        // wasm32 uses http api
        #[cfg(target_arch = "wasm32")]
        {
            "Function=".to_string()
                + &(match (input, value) {
                    (Some(input), Some(value)) => format!("{cmd}&{input}&{value}"),
                    (Some(input), None) => format!("{cmd}&{}", input),
                    (None, Some(value)) => format!("{cmd}&{value}"),
                    (None, None) => cmd.to_string(),
                })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            "FUNCTION ".to_string()
                + &(match (input, value) {
                    (Some(input), Some(value)) => format!("{cmd} {input}&{value}",),
                    (Some(input), None) => format!("{cmd} {input}"),
                    (None, Some(value)) => format!("{cmd} {value}",),
                    (None, None) => cmd.to_string(),
                })
                + "\r\n"
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
                format!("p{}s{}.Text", 1, hole)
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
            VMixPlayerInfo::Name(ind) => format!("p{}name.Text", ind + 1),
            VMixPlayerInfo::Surname(ind) => format!("p{}surname.Text", ind + 1),
            VMixPlayerInfo::TotalScore(ind) => format!("p{}scoretot.Text", ind + 1),
            VMixPlayerInfo::RoundScore(ind) => format!("p{}scorernd.Text", ind + 1),
            VMixPlayerInfo::Throw(ind) => format!("p{}throw.Text", ind + 1),
            VMixPlayerInfo::PlayerPosition(pos) => format!("p{}pos.Text", pos + 1),
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
