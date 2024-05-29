#[macro_use] extern crate rocket;
mod api;
pub mod controller;
pub mod flipup_vmix_controls;
pub mod vmix;

#[launch]
fn rocket() -> _ {
    api::launch()
}