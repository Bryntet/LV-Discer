#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};

mod api;
pub mod controller;
pub mod flipup_vmix_controls;
pub mod vmix;
mod dto;

#[launch]
fn rocket() -> Rocket<Build> {
    api::launch()
}
