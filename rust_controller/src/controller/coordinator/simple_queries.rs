use crate::controller::coordinator::FlipUpVMixCoordinator;

impl FlipUpVMixCoordinator {
    pub fn current_hole(&self) -> usize {
        self.focused_player().hole_shown_up_until
    }

    pub fn get_round(&self) -> usize {
        self.round_ind
    }
}
