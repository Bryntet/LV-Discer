pub trait VMixSelectionTrait {
    fn get_selection(&self) -> String;
    fn get_id(&self) -> String;
}
#[derive(Clone)]
pub struct VMixSelection<T: VMixSelectionTrait>(pub T);

impl<T: VMixSelectionTrait> VMixSelection<T> {
    fn get_selection(&self) -> String {
        self.0.get_selection()
    }
}

impl From<VMixProperty> for VMixSelection<VMixProperty> {
    fn from(prop: VMixProperty) -> Self {
        VMixSelection(prop)
    }
}

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
            OverlayInput4(mov) => Some(mov.to_string()),
            OverlayInput4Off | SetPanX { .. } => None,
            SetImage { input, .. } => Some(input.get_selection()),
            SetTextVisibleOn { input } => Some(input.get_selection()),
            SetTextVisibleOff { input } => Some(input.get_selection()),
        }
    }

    fn get_value(&self) -> Option<String> {
        match self {
            Self::SetText { value, .. } => Some(value.clone()),
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

#[derive(Clone)]
pub enum VMixProperty {
    Score { hole: usize, player: usize },
    HoleNumber(usize, usize),
    ScoreColor { hole: usize, player: usize },
    PosRightTriColor(usize),
    PosSquareColor(usize),
    Name(usize),
    Surname(usize),
    TotalScore(usize),
    RoundScore(usize),
    Throw(usize),
    Mov(String),
    PlayerPosition(u16),
    HoleMeters,
    HoleFeet,
    HolePar,
    Hole,
}

impl VMixSelectionTrait for VMixProperty {
    fn get_selection(&self) -> String {
        self.get_id()
            + &(match self {
                VMixProperty::Score { hole, player } => {
                    format!("SelectedName=s{}p{}.Text", hole, player + 1)
                }
                VMixProperty::HoleNumber(v1, v2) => {
                    format!("SelectedName=HN{}p{}.Text", v1, v2 + 1)
                }
                VMixProperty::ScoreColor { hole, player } => {
                    format!("SelectedName=h{}p{}.Fill.Color", hole, player + 1)
                }
                VMixProperty::PosRightTriColor(v1) => {
                    format!("SelectedName=rghtri{}.Fill.Color", v1 + 1)
                }
                VMixProperty::PosSquareColor(v1) => {
                    format!("SelectedName=rekt{}.Fill.Color", v1 + 1)
                }
                VMixProperty::Name(ind) => format!("SelectedName=namep{}.Text", ind + 1),
                VMixProperty::Surname(ind) => format!("SelectedName=surnamep{}.Text", ind + 1),
                VMixProperty::TotalScore(ind) => format!("SelectedName=scoretotp{}.Text", ind + 1),
                VMixProperty::RoundScore(ind) => format!("SelectedName=scorerndp{}.Text", ind + 1),
                VMixProperty::Throw(ind) => format!("SelectedName=t#p{}.Text", ind + 1),
                VMixProperty::Mov(id) => format!("SelectedName={}", id),
                VMixProperty::PlayerPosition(pos) => format!("SelectedName=posp{}.Text", pos + 1),

                VMixProperty::HoleMeters => "SelectedName=meternr.Text".to_string(),
                VMixProperty::HoleFeet => "SelectedName=feetnr.Text".to_string(),
                VMixProperty::HolePar => "SelectedName=parnr.Text".to_string(),
                VMixProperty::Hole => "SelectedName=hole.Text".to_string(),
            })
    }

    fn get_id(&self) -> String {
        "Input=506fbd14-52fc-495b-8d17-5b924fba64f3&".to_string()
    }
}