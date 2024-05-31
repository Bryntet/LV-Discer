use rocket::{State};
use rocket_okapi::openapi;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::vmix::functions::{VMixFunction, VMixProperty, VMixSelection};


/// # Play animation
/// Play the animation that corresponds with the upcoming score of the currently focused player
#[openapi(tag = "VMix")]
#[post("/vmix/play/animation")]
pub fn play_animation(co: &State<FlipUpVMixCoordinator>) {
    co.play_animation()
}



/// # Reset state
/// Reset the state to the default configuration.
// TODO: add leaderboard clearing
// TODO: add hole information clearing
#[openapi(tag = "VMix")]
#[post("/vmix/clear/all")]
pub fn clear_all(co: &State<FlipUpVMixCoordinator>) {
    let queue = co.queue.clone();
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

