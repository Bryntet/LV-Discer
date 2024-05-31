use crate::controller::coordinator::FlipUpVMixCoordinator;

impl FlipUpVMixCoordinator {
    pub fn current_hole(&self) -> usize {
        self.lb_thru + 1
    }

    pub fn focused_player_hole(&self) -> usize {
        self.get_focused().hole + 1
    }
    
    pub fn get_round(&self) -> usize {
        self.round_ind
    }
    
    pub fn get_rounds(&self) -> usize {
        self.get_focused().rounds.len()
    }
}