use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::get_data::HoleScoreOrDefault;

impl FlipUpVMixCoordinator {
    pub fn make_hole_info(&mut self) {
        self.set_current_through();
        if self.current_hole() <= 18 {
            self.queue_add(
                &self
                    .focused_player()
                    .results
                    .get_hole_info(self.current_through),
            );
        }
    }

    pub fn show_pos(&mut self) {
        let f = self.focused_player_mut().show_pos();
        self.queue_add(&f)
    }

    pub fn play_animation(&self) {
        let score = self.focused_player().get_score();
        self.queue_add(&score.play_mov_vmix(self.focused_player_index, false));
    }
}
