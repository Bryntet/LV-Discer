mod mutation;
mod query;
use query::*;
use rocket::{Build, Rocket, State};
use rocket::serde::json::Json;

use crate::controller::coordinator::{FlipUpVMixCoordinator, ScoreCard};

#[get("/")]
pub fn test(test: &State<FlipUpVMixCoordinator>) {
    dbg!(test.inner());
    println!("hello world");
}




pub fn launch() -> Rocket<Build> {
    rocket::build()
        .manage(FlipUpVMixCoordinator::default())
        .mount("/", routes![test,current_hole,amount_of_rounds,current_round])
}
