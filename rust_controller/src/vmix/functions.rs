pub trait VMixSelectionTrait {
    fn get_selection(&self) -> String;
    fn get_id(&self) -> &'static str;
}
#[derive(Clone)]
pub struct VMixSelection<T: VMixSelectionTrait>(pub T);

impl<T: VMixSelectionTrait> VMixSelection<T> {
    fn get_selection(&self) -> String {
        self.0.get_selection()
    }
}

impl From<LeaderBoardProperty> for VMixSelection<LeaderBoardProperty> {
    fn from(prop: LeaderBoardProperty) -> Self {
        VMixSelection(prop)
    }
}

impl From<VMixProperty> for VMixSelection<VMixProperty> {
    fn from(prop: VMixProperty) -> Self {
        VMixSelection(prop)
    }
}
pub enum LeaderBoardProperty {
    Position { pos: u16, lb_pos: u16, tied: bool },
    Name(u16),
    HotRound(u16),
    RoundScore(u16),
    TotalScore { pos: u16 },
    Move { pos: u16 },
    Arrow { pos: u16 },
    Thru(u16),
    CheckinText,
    TotalScoreTitle,
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
            Self::SetColor { color, .. } => Some("#".to_string() + color),
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
        let cmd = "FUNCTION ".to_string() + self.get_start_cmd();
        let input = self.get_input();
        let value = self.get_value();
        let command = match (input, value) {
            (Some(input), Some(value)) => format!(
                "{cmd} Input=506fbd14-52fc-495b-8d17-5b924fba64f3&{}&{}",
                input, value
            ),
            (Some(input), None) => format!("{cmd} Input={}", input),
            (None, Some(value)) => format!("{cmd} Value={}", value),
            (None, None) => cmd,
        };
        command + "\r\n"
    }
}

impl VMixSelectionTrait for LeaderBoardProperty {
    fn get_selection(&self) -> String {
        match self {
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
            LeaderBoardProperty::TotalScore { pos, .. } => format!("SelectedName=ts#{}.Text", pos),
            LeaderBoardProperty::TotalScoreTitle => "SelectedName=ts.Text".to_string(),
            LeaderBoardProperty::Move { pos, .. } => format!("SelectedName=move{}.Text", pos),
            LeaderBoardProperty::Arrow { pos, .. } => format!("SelectedName=arw{}.Source", pos),
            LeaderBoardProperty::Thru(pos) => format!("SelectedName=thru#{}.Text", pos),
            LeaderBoardProperty::CheckinText => "SelectedName=checkintext.Text".to_string(),
        }
    }

    fn get_id(&self) -> &'static str {
        "0e76d38f-6e8d-4f7d-b1a6-e76f695f2094"
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
        match self {
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
            VMixProperty::PosSquareColor(v1) => format!("SelectedName=rekt{}.Fill.Color", v1 + 1),
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
        }
    }

    fn get_id(&self) -> &'static str {
        "0e76d38f-6e8d-4f7d-b1a6-e76f695f2094"
    }
}
