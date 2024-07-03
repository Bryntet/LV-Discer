use crate::controller::coordinator::FlipUpVMixCoordinator;

impl FlipUpVMixCoordinator {
    pub fn current_hole(&self) -> usize {
        self.lb_thru + 1
    }

    pub fn focused_player_hole(&self) -> usize {
        self.focused_player().hole + 1
    }

    pub fn get_round(&self) -> usize {
        self.round_ind
    }
}
