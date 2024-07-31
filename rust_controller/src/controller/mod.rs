pub use coordinator::Player;
pub use get_data::fix_score;

pub mod coordinator;
pub(crate) mod get_data;
mod hole;
pub(crate) mod queries;
