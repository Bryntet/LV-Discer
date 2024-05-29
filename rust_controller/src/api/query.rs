

use rocket::{Rocket, Build, State};
use rocket::serde::json::Json;
use crate::controller::coordinator::FlipUpVMixCoordinator;

#[get("/current-hole")]
pub fn current_hole(coordinator: &State<FlipUpVMixCoordinator>) -> Json<usize> {
    coordinator.current_hole().into()
}

#[get("/rounds")]
pub fn amount_of_rounds(coordinator: &State<FlipUpVMixCoordinator>) -> Json<usize> {
    coordinator.get_rounds().into()
}

#[get("/round")]
pub fn current_round(coordinator: &State<FlipUpVMixCoordinator>) -> Json<usize> {
    coordinator.get_round().into()
}