use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::controller::get_data::HoleScoreOrDefault;
use crate::vmix::functions::{VMixFunction, VMixProperty};

impl FlipUpVMixCoordinator {
    pub fn make_hole_info(&mut self) {
        self.set_lb_thru();
        if self.current_hole() <= 18 {
            self.queue_add(
                &self
                    .score_card
                    .p1
                    .current_round()
                    .get_hole_info(self.lb_thru),
            );
        }
    }
    
    pub fn show_pos(&mut self) {
        let f = self.get_focused_mut().show_pos();
        self.queue_add(&f)
    }
    pub fn show_all_pos(&mut self) {
        let mut return_vec: Vec<VMixFunction<VMixProperty>> = vec![];
        return_vec.extend(self.score_card.p1.show_pos());
        return_vec.extend(self.score_card.p2.show_pos());
        return_vec.extend(self.score_card.p3.show_pos());
        return_vec.extend(self.score_card.p4.show_pos());
        self.queue_add(&return_vec);
    }

    pub fn play_animation(&self) {
        if let Some(score) = self.get_focused().get_score() {
            self.queue_add(&score.play_mov_vmix(self.foc_play_ind, false));
        }
    }
}