use std::ops::Deref;
use std::sync::{Arc, Mutex};
use crate::controller::coordinator::FlipUpVMixCoordinator;
#[derive(Default)]
struct WrappedCoordinator(Mutex<FlipUpVMixCoordinator>);


