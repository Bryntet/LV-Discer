use crate::controller::fix_score;
use crate::vmix::functions::{VMixFunction, VMixPlayerInfo, };
pub struct OverarchingScore {
    round: usize,
    round_score: isize,
    player: usize,
    total_score: isize,
}

pub enum RoundScore {
    Shown([VMixFunction<VMixPlayerInfo>; 2]),
    Hidden(VMixFunction<VMixPlayerInfo>),
}

impl OverarchingScore {
    pub fn new(round: usize, round_score: isize, player: usize, total_score: isize) -> Self {
        Self {
            round,
            round_score,
            player,
            total_score,
        }
    }
    pub fn set_round_score(&self) -> Vec<VMixFunction<VMixPlayerInfo>> {
        if self.round > 0 {
            vec![
                VMixFunction::SetText {
                    value: "(".to_string() + &fix_score(self.round_score) + ")",
                    input: VMixPlayerInfo::RoundScore(self.player).into(),
                },
                self.show_round_score(),
            ]
        } else {
            vec![self.hide_round_score()]
        }
    }
    fn hide_round_score(&self) -> VMixFunction<VMixPlayerInfo> {
        VMixFunction::SetTextVisibleOff {
            input: VMixPlayerInfo::RoundScore(self.player).into(),
        }
    }

    fn show_round_score(&self) -> VMixFunction<VMixPlayerInfo> {
        VMixFunction::SetTextVisibleOn {
            input: VMixPlayerInfo::RoundScore(self.player).into(),
        }
    }

    pub fn set_total_score(&self) -> VMixFunction<VMixPlayerInfo> {
        VMixFunction::SetText {
            value: fix_score(self.total_score),
            input: VMixPlayerInfo::TotalScore(self.player).into(),
        }
    }
}

pub enum BogeyType {
    Single,
    Double,
    Triple,
    Ouch,
}
impl BogeyType {
    const fn new(score: u8) -> Self {
        match score {
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Triple,
            _ => Self::Ouch,
        }
    }
}
pub enum ReadableScore {
    Bogey(BogeyType),
    Par,
    Birdie,
    Eagle,
    Albatross,
    Ace,
}
impl ReadableScore {
    const fn new(throws: i8, par: i8) -> Self {
        let score = throws - par;
        match score {
            0 => Self::Par,
            -1 => Self::Birdie,
            -2 => Self::Eagle,
            -3 if throws == 1 => Self::Ace,
            -3 => Self::Albatross,
            ..=-3 => Self::Ace,
            1.. => Self::Bogey(BogeyType::new(score as u8)),
        }
    }

    pub(crate) const fn to_colour(&self) -> &'static str {
        use ReadableScore::*;
        match self {
            Bogey(bogey_type) => match bogey_type {
                BogeyType::Triple | BogeyType::Ouch => "AB8E77FF",
                BogeyType::Double => "CA988DFF",
                BogeyType::Single => "EC928FFF",
            },
            Par => "7E8490FF",
            Birdie => "A6F8BBFF",
            Eagle => "6A8BE7FF",
            Ace | Albatross => "DD6AC9FF",
        }
    }

    const fn to_mov(&self) -> &'static str {
        use ReadableScore::*;
        match self {
            Bogey(bogey_type) => match bogey_type {
                BogeyType::Ouch => "40 ouch.mov",
                BogeyType::Triple => "30 3xBogey.mov",
                BogeyType::Double => "20 2xBogey.mov",
                BogeyType::Single => "10 bogey.mov",
            },
            Par => "04 par.mov",
            Birdie => "03 birdie.mov",
            Eagle => "02 eagle.mov",
            Albatross => "01 albatross.mov",
            Ace => "00 ace.mov",
        }
    }
}
pub struct Score {
    throws: i8,
    readable_score: ReadableScore,
    par: i8,
    hole: u8,
}
impl Score {
    pub(crate) const fn new(throws: i8, par: i8, hole: u8) -> Self {
        Self {
            throws,
            par,
            readable_score: ReadableScore::new(throws, par),
            hole,
        }
    }

    pub const fn par_score(&self) -> i8 {
        self.throws - self.par
    }

    pub const fn update_score_colour(&self, player: usize) -> VMixFunction<VMixPlayerInfo> {
        VMixFunction::SetColor {
            color: self.readable_score.to_colour(),
            input: VMixPlayerInfo::ScoreColor {
                hole: self.hole as usize,
                player,
            },
        }
    }
    pub fn update_score(&self, player: usize) -> [VMixFunction<VMixPlayerInfo>; 3] {
        
        [
            self.update_total_score_text(player),
            self.show_score(player),
            self.update_score_colour(player),
        ]
    }

    fn get_score_text(&self) -> String {
        let score = self.par_score();
        match score {
            (1..) => format!("%2B{}", score), // URL encoding for plus
            0 => "E".to_string(),
            _ => score.to_string(), // No need for minus as that's already a part of the score
        }
    }

    
    
    fn update_total_score_text(&self, player: usize) -> VMixFunction<VMixPlayerInfo> {
        VMixFunction::SetText {
            value: self.get_score_text(),
            input: VMixPlayerInfo::Score {
                hole: self.hole as usize, // TODO remove
                player,
            },
        }
    }

    fn show_score(&self, player: usize) -> VMixFunction<VMixPlayerInfo> {
        VMixFunction::SetTextVisibleOn {
            input: VMixPlayerInfo::Score {
                hole: self.hole as usize,
                player,
            }
        }
    }

    pub fn play_mov_vmix(&self, player: usize, ob: bool) -> [VMixFunction<VMixPlayerInfo>; 2] {
        [
            Self::stop_previous_mov(),
            //self.set_input_pan(0),
            self.to_vmix_mov(ob),
        ]
    }

    fn stop_previous_mov() -> VMixFunction<VMixPlayerInfo> {
        VMixFunction::OverlayInput4Off
    }

    fn set_input_pan(&self, player: usize) -> VMixFunction<VMixPlayerInfo> {
        let pan = match player + 1 {
            1 => -0.628,
            2 => -0.628 + 0.419,
            3 => -0.628 + 0.4185 * 2.0,
            4 => -0.628 + 0.419 * 3.0,
            _ => -0.628,
        };
        VMixFunction::SetPanX { pan }
    }

    const fn to_vmix_mov(&self, ob: bool) -> VMixFunction<VMixPlayerInfo> {
        if ob {
            VMixFunction::OverlayInput4("\"00 OB.mov\"")
        } else {
            VMixFunction::OverlayInput4(self.readable_score.to_mov())
        }
    }
}
