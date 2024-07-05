use crate::api::Coordinator;
use crate::vmix::functions::{VMixFunction, VMixProperty};
use rocket_okapi::openapi;

/// # Play animation
/// Play the animation that corresponds with the upcoming score of the currently focused player
#[openapi(tag = "VMix")]
#[post("/vmix/player/focused/animation")]
pub async fn play_animation(co: Coordinator) {
    co.lock().await.play_animation()
}

/// # Reset state
/// Reset the state to the default configuration.
// TODO: add leaderboard clearing
// TODO: add hole information clearing
#[openapi(tag = "VMix")]
#[post("/vmix/clear/all")]
pub async fn clear_all(co: Coordinator) {
    let queue = co.lock().await.queue.clone();
    let mut actions = vec![];
    for player in 0..=3 {
        for hole in 1..=9 {
            actions.extend([VMixFunction::SetText {
                value: "".to_string(),
                input: VMixProperty::Score {
                    hole,player
                }.into(),
            },VMixFunction::SetColor {
                color: "3F334D00",
                input: VMixProperty::ScoreColor {
                    hole,player
                }.into()
            }])
        }
        actions.extend([VMixFunction::SetText {
            value: "0".to_string(),
            input: VMixProperty::TotalScore(player).into()
        }, VMixFunction::SetText {
            value: "0".to_string(),
            input: VMixProperty::RoundScore(player).into()
        }])
    }
    queue.add(&actions)
}


/// # Update leaderboard
/// Set the leaderboard to the current state
#[openapi(tag = "VMix")]
#[post("/vmix/update/leaderboard")]
pub async fn update_leaderboard(co: Coordinator) {
    todo!();
}


/// # Increase score
/// Increase the score of the currently focused player
#[openapi(tag = "VMix")]
#[post("/vmix/player/focused/score/increase")]
pub async fn increase_score(co: Coordinator) {
    co.lock().await.increase_score()
}