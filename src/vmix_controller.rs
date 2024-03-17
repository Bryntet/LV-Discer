pub trait VMixSelectionTrait {
    fn get_selection(&self) -> String;
    fn get_id(&self) -> &'static str;
}
#[derive(Clone)]
pub struct VMixSelection<T: VMixSelectionTrait>(pub T);

impl<T:VMixSelectionTrait> VMixSelection<T> {
    fn get_selection(&self) -> String {
        self.0.get_selection()
    }
}

impl From<LeaderBoardProperty> for VMixSelection<LeaderBoardProperty> {
    fn from(prop: LeaderBoardProperty) -> Self {
        VMixSelection(prop)
    }
}

impl From<VmixProperty> for VMixSelection<VmixProperty> {
    fn from(prop: VmixProperty) -> Self {
        VMixSelection(prop)
    }
}
pub enum LeaderBoardProperty {
    Position{
        pos: u16,
        lb_pos: u16,
        tied: bool,
    },
    Name(u16),
    HotRound(u16),
    RoundScore(u16),
    TotalScore{
        pos: u16,
    },
    Move{
        pos: u16,
    },
    Arrow { pos: u16, },
    Thru(u16),
    CheckinText,
    TotalScoreTitle,
}


pub enum VMixFunction<InputEnum: VMixSelectionTrait> {
    SetText { value: String, input: VMixSelection<InputEnum> },
    SetPanX { value: f64, input: VMixSelection<InputEnum> },
    SetColor { color: String, input: VMixSelection<InputEnum> },
    SetTextVisibleOn { input: VMixSelection<InputEnum> },
    SetTextVisibleOff { input: VMixSelection<InputEnum> },
    SetImage { value: String, input: VMixSelection<InputEnum> },
    OverlayInput4Off,
    OverlayInput4(String),
}



impl<InputEnum: VMixSelectionTrait> VMixFunction<InputEnum> {
    fn get_input(&self) -> Option<String> {
        use VMixFunction::*;
        match self {
            SetText{input,..} => Some(input.get_selection()),
            SetColor {input, .. } => Some(input.get_selection()),
            OverlayInput4(mov) => Some(mov.to_owned()),
            OverlayInput4Off => None,
            SetImage {input, .. } => Some(input.get_selection()),
            SetPanX{input, ..} => Some(input.get_selection()),
            SetTextVisibleOn{input} => Some(input.get_selection()),
            SetTextVisibleOff{input} => Some(input.get_selection()),
        }
    }


    fn get_value(&self) -> Option<String> {
        match self {
            Self::SetText {value, .. } => Some(value.clone()),
            Self::SetColor {color, .. } => Some("#".to_string() + color),
            Self::OverlayInput4Off => None,
            Self::OverlayInput4(_) => None,
            Self::SetImage {value, .. } => Some(value.to_string()),
            Self::SetPanX{value, ..} => Some(value.to_string()),
            Self::SetTextVisibleOn{..} => None,
            Self::SetTextVisibleOff{..} => None,
        }.map(|value| "Value=".to_string() + &value)
    }

}



impl<InputEnum: VMixSelectionTrait> VMixFunction<InputEnum> {
    const fn get_start_cmd(&self) -> &'static str {
        match self {
            VMixFunction::SetText{..} => "SetText",
            VMixFunction::SetColor{..} => "SetColor",
            VMixFunction::SetPanX{..} => "SetPanX",
            VMixFunction::SetTextVisibleOn{..} => "SetTextVisibleOn",
            VMixFunction::SetTextVisibleOff{..} => "SetTextVisibleOff",
            VMixFunction::SetImage { .. } => "SetImage",
            VMixFunction::OverlayInput4Off => "OverlayInput4Off",
            VMixFunction::OverlayInput4(_) => "OverlayInput4",
        }
    }

    pub fn to_cmd(&self) -> String {
        let cmd = "FUNCTION ".to_string() + self.get_start_cmd();
        let input = self.get_input();
        let value = self.get_value();
        match (input,value) {
            (Some(input),Some(value)) => format!("{cmd} Input={}&Value={}",input,value),
            (Some(input), None) => format!("{cmd} Input={}",input),
            (None,Some(value)) => format!("{cmd} Value={}",value),
            (None,None) => cmd
        }
    }
}






impl VMixSelectionTrait for LeaderBoardProperty {
    fn get_selection(&self) -> String {
        match self {
            LeaderBoardProperty::Position{pos, lb_pos, tied} => {
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
            },
            LeaderBoardProperty::Name(pos) => format!("SelectedName=name#{}.Text", pos),
            LeaderBoardProperty::HotRound(pos) => format!("SelectedName=hrp{}.Source", pos),
            LeaderBoardProperty::RoundScore(pos) => format!("SelectedName=rs#{}.Text", pos),
            LeaderBoardProperty::TotalScore{pos, ..} => format!("SelectedName=ts#{}.Text", pos),
            LeaderBoardProperty::TotalScoreTitle => "SelectedName=ts.Text".to_string(),
            LeaderBoardProperty::Move{pos,..} => format!("SelectedName=move{}.Text", pos),
            LeaderBoardProperty::Arrow{pos,..} => format!("SelectedName=arw{}.Source", pos),
            LeaderBoardProperty::Thru(pos) => format!("SelectedName=thru#{}.Text", pos),
            LeaderBoardProperty::CheckinText => "SelectedName=checkintext.Text".to_string(),
        }
    }
    
    fn get_id(&self) -> &'static str {
        "0e76d38f-6e8d-4f7d-b1a6-e76f695f2094"
    }
}

#[derive(Clone)]
pub enum VmixProperty {
    Score(usize, usize),
    HoleNumber(usize, usize),
    ScoreColor(usize, usize),
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

impl VMixSelectionTrait for VmixProperty {
    fn get_selection(&self) -> String {
        match self {
            VmixProperty::Score(v1, v2) => format!("SelectedName=s{}p{}.Text", v1, v2 + 1),
            VmixProperty::HoleNumber(v1, v2) => {
                format!("SelectedName=HN{}p{}.Text", v1, v2 + 1)
            }
            VmixProperty::ScoreColor(v1, v2) => {
                format!("SelectedName=h{}p{}.Fill.Color", v1, v2 + 1)
            }
            VmixProperty::PosRightTriColor(v1) => {
                format!("SelectedName=rghtri{}.Fill.Color", v1 + 1)
            }
            VmixProperty::PosSquareColor(v1) => format!("SelectedName=rekt{}.Fill.Color", v1 + 1),
            VmixProperty::Name(ind) => format!("SelectedName=namep{}.Text", ind + 1),
            VmixProperty::Surname(ind) => format!("SelectedName=surnamep{}.Text", ind + 1),
            VmixProperty::TotalScore(ind) => format!("SelectedName=scoretotp{}.Text", ind + 1),
            VmixProperty::RoundScore(ind) => format!("SelectedName=scorerndp{}.Text", ind + 1),
            VmixProperty::Throw(ind) => format!("SelectedName=t#p{}.Text", ind + 1),
            VmixProperty::Mov(id) => format!("SelectedName={}", id),
            VmixProperty::PlayerPosition(pos) => format!("SelectedName=posp{}.Text", pos + 1),


            VmixProperty::HoleMeters => "SelectedName=meternr.Text".to_string(),
            VmixProperty::HoleFeet => "SelectedName=feetnr.Text".to_string(),
            VmixProperty::HolePar => "SelectedName=parnr.Text".to_string(),
            VmixProperty::Hole => "SelectedName=hole.Text".to_string(),
        }
    }

    fn get_id(&self) -> &'static str {
        
        "0e76d38f-6e8d-4f7d-b1a6-e76f695f2094"
    }
}
