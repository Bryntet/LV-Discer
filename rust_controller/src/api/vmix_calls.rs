use crate::api::{Coordinator, Error, GeneralChannel, HoleUpdate};
use crate::vmix::functions::{VMixFunction, VMixInterfacer, VMixPlayerInfo};
use rocket::State;
use rocket_okapi::openapi;

/// # Play animation
/// Play the animation that corresponds with the upcoming score of the currently focused player
#[openapi(tag = "VMix")]
#[post("/vmix/player/focused/animation/play")]
pub async fn play_animation(co: Coordinator) -> Result<(), Error> {
    co.lock().await.play_animation()
}

/// # Update leaderboard
/// Set the leaderboard to the current state
#[openapi(tag = "VMix")]
#[post("/vmix/update/leaderboard/<division>")]
pub async fn update_leaderboard(co: Coordinator, division: &str) -> Result<(), Error> {
    let mut co = co.lock().await;

    let div = co
        .all_divs
        .iter()
        .find(|div| div.name == division)
        .ok_or(Error::InvalidDivision(division.to_string()))?
        .to_owned();
    co.set_leaderboard(&div, None);
    Ok(())
}

/// # Increase score
/// Increase the score of the currently focused player
#[openapi(tag = "VMix")]
#[post("/vmix/player/focused/score/increase")]
pub async fn increase_score(
    co: Coordinator,
    hole_update: &State<GeneralChannel<HoleUpdate>>,
) -> Result<(), Error> {
    let mut coordinator = co.lock().await;
    coordinator.increase_score(hole_update)?;
    Ok(())
}

/// # Revert score
/// Revert the score of the currently focused player
#[openapi(tag = "VMix")]
#[post("/vmix/player/focused/score/revert")]
pub async fn revert_score(co: Coordinator) {
    co.lock().await.revert_score()
}

/// # Increase throw
/// Increase the throw count of the currently focused player
#[openapi(tag = "VMix")]
#[post("/vmix/player/focused/throw/increase")]
pub async fn increase_throw(co: Coordinator) {
    co.lock().await.increase_throw()
}

/// # Revert throw
/// Revert the throw count of the currently focused player
#[openapi(tag = "VMix")]
#[post("/vmix/player/focused/throw/decrease")]
pub async fn revert_throw(co: Coordinator) {
    co.lock().await.decrease_throw()
}

/// # Play OB animation
/// Play the out-of-bounds animation
#[openapi(tag = "VMix")]
#[post("/vmix/player/focused/animation/play/ob")]
pub async fn play_ob_animation(co: Coordinator) -> Result<(), Error> {
    co.lock().await.ob_anim()
}

/// # Set hole info
/// Set the hole information
#[openapi(tag = "VMix")]
#[post("/vmix/hole-info/set")]
pub async fn set_hole_info(co: Coordinator) {
    co.lock().await.make_hole_info()
}

/// # Update other leaderboard
/// Update the leaderboard for a specific division
#[openapi(tag = "VMix")]
#[post("/vmix/leaderboard/<division>/update")]
pub async fn update_other_leaderboard(co: Coordinator, division: &str) -> Result<(), &'static str> {
    let coordinator = co.lock().await;
    let division = coordinator
        .find_division(division)
        .ok_or("Unable to find division")?;
    //coordinator.set_leaderboard(&division, None);
    //coordinator.make_separate_lb(&division);
    Ok(())
}
