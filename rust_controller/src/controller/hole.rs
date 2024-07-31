use itertools::Itertools;
use rayon::prelude::*;

use crate::vmix::functions::VMixSelectionTrait;

use super::queries::HoleResult;

pub struct HoleStats {
    hole_number: u8,
    player_results: Vec<HoleResult>,
}

impl HoleStats {
    pub fn new(hole_number: u8, player_results: Vec<HoleResult>) -> Self {
        Self {
            hole_number,
            player_results,
        }
    }
    pub fn average_score(&self) -> f64 {
        (self
            .player_results
            .par_iter()
            .map(|res| res.score)
            .sum::<f64>()
            / (self.player_results.len() as f64)
            * 10.)
            .round()
            / 10.
    }
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
        difficulty: HoleDifficulty,
    },
    Elevation(i16),
}
#[derive(Clone, Debug)]
pub struct HoleDifficulty {
    holes: Vec<u8>,
}

impl HoleDifficulty {
    pub fn new(holes: Vec<HoleStats>) -> Self {
        Self {
            holes: holes
                .iter()
                .sorted_by_key(|hole| hole.hole_number)
                .map(|hole| {
                    holes
                        .iter()
                        .filter(|other_hole| hole.average_score() <= other_hole.average_score())
                        .count() as u8
                })
                .collect(),
        }
    }

    fn hole_difficulty_text(&self, hole: usize) -> Option<String> {
        let difficulty = self.holes.get(hole)?;
        Some(match difficulty {
            1 => "EASIEST".to_string(),
            2 => "2nd easiest".to_string(),
            3 => "3rd easiest".to_string(),
            4..=9 => format!("{difficulty}th easiest"),
            10..=15 => format!("{}th hardest", difficulty - 9),
            16 => "3rd hardest".to_string(),
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
