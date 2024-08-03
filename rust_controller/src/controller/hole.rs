use std::cmp::Ordering;

use itertools::Itertools;
use rayon::prelude::*;

use crate::vmix::functions::VMixSelectionTrait;

use super::queries::HoleResult;

pub struct HoleStats {
    pub hole_number: u8,
    player_results: Vec<HoleResult>,
}

impl HoleStats {
    pub fn new(hole_number: u8, player_results: Vec<HoleResult>) -> Self {
        Self {
            hole_number,
            player_results,
        }
    }
    pub fn average_score(&self) -> (isize, std::cmp::Ordering) {
        let avg_result = self
            .player_results
            .par_iter()
            .map(|res| res.score)
            .sum::<f64>()
            / (self.player_results.len() as f64);
        let cmp = avg_result.total_cmp(&0.);
        (
            (self
                .player_results
                .par_iter()
                .map(|res| res.score)
                .sum::<f64>()
                / (self.player_results.len() as f64)
                * 10.)
                .round() as isize
                - self
                    .player_results
                    .first()
                    .map(|res| res.hole.par)
                    .unwrap()
                    .unwrap() as isize
                    * 10,
            cmp,
        )
    }
}
#[derive(Clone, Debug)]
pub enum VMixHoleInfo {
    Hole(u8),
    HolePar(u8),
    HoleMeters(u16),
    HoleFeet(u16),
    AverageResult {
        score: isize,
        cmp: std::cmp::Ordering,
    },
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
                        .filter(|other_hole| hole.average_score().0 <= other_hole.average_score().0)
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
            10..=15 => format!("{}th hardest", 18 - difficulty),
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
            AverageResult { .. } => "avgresult",
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
            AverageResult { score, cmp } => {
                let score = score.to_string();
                let mut all_nums = score.chars().rev();
                let decimal = all_nums.next().unwrap();
                let mut rest: String = all_nums.rev().collect();
                if rest.is_empty() || rest.contains("-") && rest.len() == 1 {
                    rest.push('0');
                }
                (match cmp {
                    Ordering::Less => format!("{rest}.{decimal}"),
                    Ordering::Equal => "E".to_string(),
                    Ordering::Greater => format!("%2B{rest}.{decimal}"),
                }) + " avg"
            }
            Difficulty { difficulty, hole } => difficulty.hole_difficulty_text(*hole).unwrap(),
            Elevation(elevation) => {
                let sign = if elevation.is_positive() { "+" } else { "" };
                format!("{sign}{elevation}")
            }
        })
    }
    const INPUT_ID: &'static str = "d9806a48-8766-40e0-b7fe-b217f9b1ef5b";
}

pub struct FeaturedHole(pub VMixHoleInfo);
impl VMixSelectionTrait for FeaturedHole {
    fn get_selection_name(&self) -> String {
        self.0.get_selection_name()
    }

    fn data_extension(&self) -> &'static str {
        self.0.data_extension()
    }

    fn value(&self) -> Option<String> {
        self.0.value()
    }

    const INPUT_ID: &'static str = "0e9bec31-70a9-4566-a1ea-e050434c1cd2";
}

pub enum DroneHoleInfo {
    Standard(VMixHoleInfo),
    HoleMap,
}
impl VMixSelectionTrait for DroneHoleInfo {
    fn get_selection_name(&self) -> String {
        match self {
            DroneHoleInfo::Standard(s) => s.get_selection_name(),
            DroneHoleInfo::HoleMap => "holemap".to_string(),
        }
    }

    fn data_extension(&self) -> &'static str {
        match self {
            DroneHoleInfo::Standard(s) => s.data_extension(),
            DroneHoleInfo::HoleMap => "Source",
        }
    }

    fn value(&self) -> Option<String> {
        match self {
            DroneHoleInfo::Standard(s) => s.value(),
            DroneHoleInfo::HoleMap => None,
        }
    }

    const INPUT_ID: &'static str = "d135d6d1-11ee-4169-9700-4c743d729218";
}
