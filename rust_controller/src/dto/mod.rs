mod coordinator_builder;
mod group;
mod player;
mod rounds;

use serde::{Serialize};
pub use coordinator_builder::CoordinatorBuilder;
pub use group::Group;
pub use player::*;
pub use rounds::{get_rounds, Rounds};

#[derive(Debug,Clone,Serialize)]
pub struct Division {
    pub name: String,
    pub focused: bool
}