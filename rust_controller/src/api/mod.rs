mod query;
mod mutation;

use rocket::{Build, Rocket, State};

use crate::controller::coordinator::{FlipUpVMixCoordinator, ScoreCard};

#[get("/")]
pub fn test(test: &State<FlipUpVMixCoordinator>) {
    dbg!(test.inner());
    println!("hello world");
}

pub fn launch() -> Rocket<Build> {
    rocket::build().manage(FlipUpVMixCoordinator::default()).mount("/",routes![test])
}