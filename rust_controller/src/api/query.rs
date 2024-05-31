use rocket::{Rocket, Build, State};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use crate::controller::coordinator::FlipUpVMixCoordinator;
/// # GET current hole
#[openapi(tag = "Hole")]
#[get("/current-hole")]
pub fn current_hole(coordinator: &State<FlipUpVMixCoordinator>) -> Json<usize> {
    coordinator.current_hole().into()
}
/// # GET Amount of rounds
#[openapi(tag = "Round")]
#[get("/rounds")]
pub fn amount_of_rounds(coordinator: &State<FlipUpVMixCoordinator>) -> Json<usize> {
    coordinator.get_rounds().into()
}

/// # GET Current round
#[openapi(tag = "Round")]
#[get("/round")]
pub fn current_round(coordinator: &State<FlipUpVMixCoordinator>) -> Json<usize> {
    coordinator.get_round().into()
}