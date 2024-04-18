use crate::get_data::fix_score;
use crate::vmix::functions::{VMixFunction, VMixProperty, VMixSelection};



pub struct OverarchingScore {
    round: usize,
    round_score: isize,
    player: usize,
    total_score: isize
}

pub enum RoundScore {
    Shown([VMixFunction<VMixProperty>;2]),
    Hidden(VMixFunction<VMixProperty>)
}

impl OverarchingScore {
    pub fn new(round: usize, round_score: isize, player: usize, total_score: isize) -> Self {
        Self {
            round,
            round_score,
            player,
            total_score
        }
    }
    pub fn set_round_score(&self) -> RoundScore {
        if self.round > 0 {
            RoundScore::Shown([
                VMixFunction::SetText {
                    value: "(".to_string() + &crate::get_data::fix_score(self.round_score) + ")",
                    input: VMixProperty::RoundScore(self.player).into(),
                },
                self.show_round_score(),
            ])
        } else {
            RoundScore::Hidden(self.hide_round_score())
        }
    }
    fn hide_round_score(&self) -> VMixFunction<VMixProperty> {
        VMixFunction::SetTextVisibleOff {
            input: VMixProperty::RoundScore(self.player).into(),
        }
    }

    fn show_round_score(&self) -> VMixFunction<VMixProperty> {
        VMixFunction::SetTextVisibleOn {
            input: VMixProperty::RoundScore(self.player).into(),
        }
    }
    
    pub fn set_total_score(&self) -> VMixFunction<VMixProperty> {
        VMixFunction::SetText {
            value: fix_score(self.total_score),
            input: VMixProperty::TotalScore(self.player).into(),
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
    const fn new(throws: usize, par: usize) -> Self {
        let score = throws - par;
        match score as i8 {
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
    throws: usize,
    readable_score: ReadableScore,
    par: usize,
    hole: usize
}
impl Score {
    pub(crate) const fn new(throws: usize, par: usize, hole:usize) -> Self {
        Self {
            throws,
            par,
            readable_score: ReadableScore::new(throws, par),
            hole
        }
    }

    pub const fn to_vmix_colour_update(
        &self,
        player: usize,
    ) -> VMixFunction<VMixProperty> {
        VMixFunction::SetColor {
            color: self.readable_score.to_colour(),
            input: VMixSelection(VMixProperty::ScoreColor { hole:self.hole, player }),
        }
    }
    pub fn to_vmix_score_update(&self, player: usize,) -> [VMixFunction<VMixProperty>; 3] {
        [self.to_vmix_hole_score_text_update(player),self.to_vmix_show_hole_score_text(player),self.to_vmix_colour_update(player)]
    }

    fn to_vmix_hole_score_text_update(&self, player:usize) -> VMixFunction<VMixProperty> {
        VMixFunction::SetText {
            value: (self.throws - self.par).to_string(),
            input: VMixSelection(VMixProperty::Score { hole:self.hole, player }),
        }
    }

    fn to_vmix_show_hole_score_text(&self, player:usize) -> VMixFunction<VMixProperty> {
        VMixFunction::SetTextVisibleOn {
            input: VMixSelection(VMixProperty::Score {
                hole: self.hole,
                player,
            }),
        }
    }

    pub fn play_mov_vmix(&self, player: usize, ob: bool) -> [VMixFunction<VMixProperty>;3] {
        [Self::stop_previous_mov(),self.set_input_pan(player),self.to_vmix_mov(ob)]
    }

    fn stop_previous_mov() -> VMixFunction<VMixProperty> {
        VMixFunction::OverlayInput4Off
    }

    fn set_input_pan(&self, player: usize) -> VMixFunction<VMixProperty> {
        let pan = match player + 1 {
            1 => -0.628,
            2 => -0.628 + 0.419,
            3 => -0.628 + 0.4185 * 2.0,
            4 => -0.628 + 0.419 * 3.0,
            _ => -0.628,
        };
        VMixFunction::SetPanX {
            pan,
        }
    }

    const fn to_vmix_mov(&self, ob:bool) -> VMixFunction<VMixProperty> {
        if ob {
            VMixFunction::OverlayInput4("00 OB.mov")
        } else {
            VMixFunction::OverlayInput4(self.readable_score.to_mov())
        }
    }
}