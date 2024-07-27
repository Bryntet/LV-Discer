#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};

mod api;
pub mod controller;
mod dto;
pub mod flipup_vmix_controls;
pub mod vmix;

#[launch]
fn rocket() -> Rocket<Build> {
    api::launch()
}
