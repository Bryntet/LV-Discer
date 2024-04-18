use crate::queries::SimpleResult;

struct RoundStats<'results> {
    round: u8,
    holes: [HoleStats<'results>; 18],
}

impl<'results> RoundStats<'results> {
    fn new(round_number: u8, holes: [(u8, &'results [SimpleResult]); 18]) -> Self {
        Self {
            round: round_number,
            holes: holes.map(|(hole_number, results)| HoleStats::new(hole_number, results)),
        }
    }
}

struct HoleStats<'results> {
    hole_number: u8,
    player_results: &'results [SimpleResult],
}

impl<'results> HoleStats<'results> {
    fn new(hole_number: u8, player_results: &'results [SimpleResult]) -> Self {
        Self {
            hole_number,
            player_results,
        }
    }
    pub fn average_score(&self) -> i8 {
        (self.player_results.iter().map(|res| res.score).sum::<f64>()
            / (self.player_results.len() as f64))
            .round() as i8
    }
}
