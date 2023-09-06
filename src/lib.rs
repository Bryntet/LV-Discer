mod get_data;
mod utils;
use js_sys::JsString;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    log("Hello, wasm-test!");
}

#[wasm_bindgen]
pub fn test() -> ScoreCard {
    ScoreCard::default()
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Constants {
    ip: String,
    default_bg_col: String,
    vmix_id: String,
}
impl Default for Constants {
    fn default() -> Self {
        Self {
            ip: "192.168.120.135".to_string(),
            default_bg_col: "3F334D".to_string(),
            vmix_id: "506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct MyApp {
    #[wasm_bindgen(skip)]
    pub all_divs: Vec<get_data::queries::PoolLeaderboardDivision>,
    #[wasm_bindgen(getter_with_clone)]
    pub score_card: ScoreCard,
    selected_div_ind: usize,
    #[wasm_bindgen(skip)]
    pub selected_div: cynic::Id,
    foc_play_ind: usize,
    consts: Constants,
    event_id: String,
    pools: Vec<get_data::queries::Pool>,
    handler: Option<get_data::RustHandler>,
    available_players: Vec<get_data::NewPlayer>,
    round_ind: usize,
    lb_div_ind: usize,
    lb_thru: usize,
    lb_vmix_id: String,
}

impl Default for MyApp {
    fn default() -> MyApp {
        MyApp {
            score_card: ScoreCard::default(),
            all_divs: vec![],
            selected_div_ind: 0,
            selected_div: cynic::Id::from("cd094fcf-76c7-471e-bfe3-be3d5892bd81"),
            foc_play_ind: 0,
            consts: Constants::default(),
            pools: vec![],
            event_id: "75cceb0e-5a1d-4fba-a5c8-f2ff95f84495".into(),
            handler: None,
            available_players: vec![],
            round_ind: 0,
            lb_div_ind: 0,
            lb_thru: 0,
            lb_vmix_id: "2ef7178b-61ab-445c-9bbd-2f1c2c781e86".into(),
        }
    }
}

#[wasm_bindgen]
impl MyApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> MyApp {
        utils::set_panic_hook();
        MyApp::default()
    }
    #[wasm_bindgen(setter = ip)]
    pub fn set_ip(&mut self, ip: String) {
        self.consts.ip = ip.clone();
        self.score_card.consts.ip = ip;
        log(&format!("ip set to {}", &self.consts.ip));
    }
    #[wasm_bindgen(setter = div)]
    pub fn set_div(&mut self, idx: usize) {
        log(&format!("div set to {}", idx));
        self.selected_div_ind = idx;
        if let Some(handler) = &mut self.handler {
            handler.set_chosen_by_ind(idx);
        }
        log("did that");
        self.get_players(false);
        log("here now hah");
        // self.selected_div = Some(self.all_divs[idx].clone());
        // self.score_card.all_play_players = self.selected_div.as_ref().unwrap().players.clone();
    }

    #[wasm_bindgen]
    pub fn set_leaderboard(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        return_vec.append(&mut MyApp::clear_lb(10));
        log("set_leaderboard");
        //let mut lb_copy = self.clone();
        self.set_lb_thru();
        log("past set_lb_thru");
        if self.get_hole(false) <= 19 {
            log("hole <= 19");
            self.lb_div_ind = self.selected_div_ind;
            self.get_players(true);
            log(&format!(
                "{:#?}",
                self.score_card
                    .all_play_players
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<String>>()
            ));
            log("past get_players");
            self.fix_players();
            log("past fix_players");
            return_vec.append(&mut self.make_lb());

            let players = [
                &self.score_card.p1,
                &self.score_card.p2,
                &self.score_card.p3,
                &self.score_card.p4,
            ];
            for player in players {
                let same = self.find_same(player).unwrap();
                log(&format!("same: {:#?}", same.set_pos()));
                let mut cloned_player = player.clone();
                if cloned_player.hole > 7 {
                    cloned_player.hole -= 1;
                    return_vec.append(&mut cloned_player.shift_scores(true));
                }
                return_vec.push(same.set_pos());
            }
            // return_vec.push(self.find_same(&self.score_card.p1).unwrap().set_pos());
            // return_vec.push(self.find_same(&self.score_card.p2).unwrap().set_pos());
            // return_vec.push(self.find_same(&self.score_card.p3).unwrap().set_pos());
            // return_vec.push(self.find_same(&self.score_card.p4).unwrap().set_pos());
        } else {
            log("PANIC, hole > 18");
        }
        return_vec
    }

    pub fn clear_lb(idx: u16) -> Vec<JsString> {
        let mut new_player = get_data::NewPlayer::default();
        new_player.lb_vmix_id = "2ef7178b-61ab-445c-9bbd-2f1c2c781e86".into();
        new_player.hidden = true;
        let mut r_v: Vec<JsString> = vec![];
        for i in 0..=idx {
            new_player.position = i;
            r_v.append(&mut new_player.set_lb());
        }
        r_v
    }

    #[wasm_bindgen]
    pub fn set_all_to_hole(&mut self, hole: usize) -> Vec<JsString> {
        vec![
            &mut self.score_card.p1,
            &mut self.score_card.p2,
            &mut self.score_card.p3,
            &mut self.score_card.p4,
        ]
        .iter_mut()
        .flat_map(|player| {
            if hole >= 9 {
                player.hole = hole - 1;
                player.shift_scores(true)
            } else {
                let mut r_vec: Vec<JsString> = vec![];
                for x in 1..=hole {
                    log(&format!("hello im here: {}", x));
                    player.hole = x;
                    r_vec.append(&mut player.set_hole_score());
                }
                r_vec
            }
        })
        .collect()
    }

    fn find_same(&self, player: &get_data::NewPlayer) -> Option<get_data::NewPlayer> {
        for p in &self.score_card.all_play_players {
            if p.player_id == player.player_id {
                return Some({
                    let mut cl = p.clone();
                    cl.ind = player.ind;
                    cl
                });
            }
        }
        None
    }

    // #[wasm_bindgen(setter = lb_div)]
    // pub fn set_lb_div(&mut self, idx: usize) {
    //     self.lb_div_ind = idx;
    //     self.handler
    //         .clone()
    //         .expect("handler!")
    //         .set_chosen_by_ind(idx);
    //     self.get_players(true);
    //     self.fix_players();
    // }

    fn set_lb_thru(&mut self) {
        let focused_players = vec![
            &self.score_card.p1,
            &self.score_card.p2,
            &self.score_card.p3,
            &self.score_card.p4,
        ];
        self.lb_thru = focused_players.iter().map(|p| p.hole).min().unwrap_or(0);
    }

    #[wasm_bindgen]
    pub fn get_hole(&mut self, check_thru: bool) -> usize {
        if check_thru {
            self.set_lb_thru();
        }
        self.lb_thru + 1
    }

    #[wasm_bindgen(getter = focused_player_hole)]
    pub fn focused_player_hole(&mut self) -> usize {
        self.get_focused().hole + 1
    }
    #[wasm_bindgen(getter = hole)]
    pub fn get_hole_js(&self) -> usize {
        self.lb_thru + 1
    }

    fn fix_players(&mut self) {
        if self.round_ind != 0 {
            for player in &mut self.score_card.all_play_players {
                player.old_pos = player.lb_pos;
                player.set_round(self.round_ind - 1);
                player.hole = 17;
                player.make_tot_score();
            }
            self.assign_position();
            self.set_hot_round();
        }
        for player in &mut self.score_card.all_play_players {
            player.hot_round = false;
            player.lb_even = false;
            player.old_pos = player.lb_pos;
            player.set_round(self.round_ind);
            //log(&format!("player.hole: {}", player.hole));
            player.thru = self.lb_thru as u8;
            player.hole = if self.lb_thru > 0 {
                self.lb_thru - 1
            } else {
                0
            };
            //log(&format!("player.hole after: {}", player.hole));
            player.make_tot_score();
        }
        self.assign_position();
        self.set_hot_round();
    }

    pub fn assign_position(&mut self) {
        // Sort players in descending order by total_score

        self.score_card.all_play_players.sort_unstable_by(|a, b| {
            {
                if !a.dnf {
                    a.total_score
                } else {
                    i16::MIN
                }
            }
            .cmp(if !b.dnf { &b.total_score } else { &i16::MIN })
        });

        // Iterate over sorted players to assign position

        let play_len = self.score_card.all_play_players.len();
        log(&format!("Length of play_len: {play_len}"));
        for i in 0..play_len {
            let mut next_play = None;
            let mut prev_play = None;

            if i != 0 {
                prev_play = Some(self.score_card.all_play_players[i - 1].clone());
            }
            if i + 1 < play_len {
                next_play = Some(self.score_card.all_play_players[i + 1].clone());
            }

            let player = &mut self.score_card.all_play_players[i];
            player.position = i as u16 + 1;

            if let Some(next_play) = next_play.clone() {
                if let Some(prev_play) = prev_play.clone() {
                    if player.total_score != prev_play.total_score {
                        player.lb_pos = i as u16 + 1;
                    } else {
                        player.lb_pos = prev_play.lb_pos;
                    }
                } else {
                    player.lb_pos = i as u16 + 1;
                }
                if player.total_score == next_play.total_score {
                    player.lb_even = true
                }
            } else {
                player.lb_pos = play_len as u16
            }
            if let Some(prev_play) = prev_play {
                if player.total_score == prev_play.total_score {
                    if next_play.is_none() {
                        player.lb_pos = prev_play.lb_pos
                    }
                    player.lb_even = true;
                }
            }
            player.check_pos();
        }
    }

    fn make_checkin_text(&self) -> JsString {
        get_data::VmixFunction::SetText(get_data::VmixInfo {
            id: &self.lb_vmix_id,
            value: String::from(self.get_div_names()[self.selected_div_ind].to_string())
                .to_uppercase()
                + " "
                + "LEADERBOARD CHECK-IN",
            prop: get_data::VmixProperty::LBCheckinText,
        })
        .to_cmd()
        .into()
    }

    fn set_hot_round(&mut self) {
        let hot_round = self
            .score_card
            .all_play_players
            .iter()
            .map(|player| player.round_score)
            .min()
            .unwrap_or(0);
        for player in &mut self.score_card.all_play_players {
            if player.round_score == hot_round {
                player.hot_round = true;
            }
        }
    }

    #[wasm_bindgen]
    pub fn make_hole_info(&mut self) -> Vec<JsString> {
        if self.get_hole(true) <= 18 {
            self.score_card
                .p1
                .current_round()
                .get_hole_info(self.lb_thru)
        } else {
            vec![]
        }
    }

    pub fn make_lb(&mut self) -> Vec<JsString> {
        let mut r_vec: Vec<JsString> = self
            .score_card
            .all_play_players
            .iter_mut()
            .flat_map(|player| player.set_lb())
            .collect();
        r_vec.push(self.make_checkin_text());
        r_vec
    }

    #[wasm_bindgen(getter = round)]
    pub fn get_round(&self) -> usize {
        self.round_ind
    }

    #[wasm_bindgen]
    pub fn set_round(&mut self, idx: usize) -> Vec<JsString> {
        self.round_ind = idx;
        self.lb_thru = 0;
        self.score_card.set_round(idx)
    }

    #[wasm_bindgen(getter = rounds)]
    pub fn get_rounds(&mut self) -> usize {
        self.get_focused().rounds.len()
    }

    #[wasm_bindgen]
    pub async fn get_event(&mut self) -> Result<JsValue, JsValue> {
        self.pools = vec![];
        let promise: usize = 0;
        self.handler = Some(get_data::RustHandler::new(
            get_data::post_status(cynic::Id::from(&self.event_id)).await,
            self.consts.vmix_id.clone(),
            self.lb_vmix_id.clone(),
        ));

        match self.handler.clone() {
            Some(..) => log("handler fine"),
            None => log("handler on fire"),
        }
        let promise = js_sys::Promise::resolve(&JsValue::from_str(&promise.to_string()));
        let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(result)
    }

    // #[wasm_bindgen]
    // pub async fn get_divs(&mut self) -> Result<JsValue, JsValue> {
    //     self.all_divs = vec![];

    //     if let Some(pool) = self.pool {
    //         if let Some(lb) = pool.leaderboard {
    //             for div in lb {
    //                 if let Some(div) = div {
    //                     match div {
    //                         get_data::queries::PoolLeaderboardDivisionCombined::PoolLeaderboardDivision(d) => {
    //                             self.all_divs.push(d);
    //                         }
    //                         _ => {}
    //                     }
    //                 }
    //             }
    //         }
    //     }

    // }

    #[wasm_bindgen]
    pub fn increase_score(&mut self) -> Vec<JsString> {
        let hole = self.get_focused().hole;
        let mut out: Vec<JsString> = self.hide_pos();
        //log(format!("hole: {}", hole).as_str());
        if hole <= 17 {
            if hole <= 7 {
                out.append(&mut self.get_focused().set_hole_score());
                out
            } else {
                out.append(&mut self.get_focused().shift_scores(false));
                out
            }
        } else {
            vec![]
        }
    }

    #[wasm_bindgen]
    pub fn show_pos(&mut self) -> Vec<JsString> {
        self.get_focused().show_pos()
    }

    #[wasm_bindgen]
    pub fn show_all_pos(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        return_vec.append(&mut self.score_card.p1.show_pos());
        return_vec.append(&mut self.score_card.p2.show_pos());
        return_vec.append(&mut self.score_card.p3.show_pos());
        return_vec.append(&mut self.score_card.p4.show_pos());
        return_vec
    }

    #[wasm_bindgen]
    pub fn hide_pos(&mut self) -> Vec<JsString> {
        self.get_focused().hide_pos()
    }

    #[wasm_bindgen]
    pub fn hide_all_pos(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        return_vec.append(&mut self.score_card.p1.hide_pos());
        return_vec.append(&mut self.score_card.p2.hide_pos());
        return_vec.append(&mut self.score_card.p3.hide_pos());
        return_vec.append(&mut self.score_card.p4.hide_pos());
        return_vec
    }

    pub fn toggle_pos(&mut self) -> Vec<JsString> {
        self.get_focused().toggle_pos()
    }

    #[wasm_bindgen]
    pub fn play_animation(&mut self) -> Vec<JsString> {
        log("play_animation");
        if self.get_focused().hole <= 17 {
            self.get_focused().start_score_anim()
        } else {
            vec![]
        }
    }

    #[wasm_bindgen]
    pub fn ob_anim(&mut self) -> Vec<JsString> {
        log("ob_anim");
        self.get_focused().ob = true;
        self.get_focused().start_score_anim()
    }

    fn get_focused(&mut self) -> &mut get_data::NewPlayer {
        match self.foc_play_ind {
            0 => &mut self.score_card.p1,
            1 => &mut self.score_card.p2,
            2 => &mut self.score_card.p3,
            3 => &mut self.score_card.p4,
            _ => &mut self.score_card.p1,
        }
    }

    #[wasm_bindgen]
    pub fn set_player(&mut self, idx: usize, player: JsString) -> Vec<JsString> {
        self.score_card.set_player(idx, player, self.round_ind)
    }

    #[wasm_bindgen]
    pub fn set_foc(&mut self, idx: usize) {
        self.foc_play_ind = idx;
    }
    #[wasm_bindgen]
    pub fn revert_score(&mut self) -> Vec<JsString> {
        self.get_focused().revert_hole_score()
    }
    #[wasm_bindgen]
    pub fn reset_score(&mut self) -> Vec<JsString> {
        self.lb_thru = 0;
        self.get_focused().reset_scores()
    }

    #[wasm_bindgen]
    pub fn reset_scores(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        return_vec.append(&mut self.score_card.p1.reset_scores());
        return_vec.append(&mut self.score_card.p2.reset_scores());
        return_vec.append(&mut self.score_card.p3.reset_scores());
        return_vec.append(&mut self.score_card.p4.reset_scores());
        return_vec.append(&mut MyApp::clear_lb(10));
        return_vec
    }

    #[wasm_bindgen]
    pub fn get_foc_p_name(&mut self) -> JsString {
        self.get_focused().name.clone().into()
    }

    #[wasm_bindgen]
    pub fn get_div_names(&self) -> Vec<JsString> {
        let mut return_vec = vec![];

        for div in self.handler.clone().expect("handler!").get_divisions() {
            return_vec.push(div.name.clone().into());
        }
        return_vec
    }

    #[wasm_bindgen]
    pub fn get_players(&mut self, lb: bool) {
        self.available_players = self.handler.clone().expect("handler!").get_players();
        if !lb {
            self.score_card.all_play_players = self.available_players.clone();
        }
    }
    #[wasm_bindgen]
    pub fn get_player_names(&self) -> Vec<JsString> {
        self.available_players
            .iter()
            .map(|player| player.name.clone().into())
            .collect()
    }

    #[wasm_bindgen]
    pub fn get_player_ids(&self) -> Vec<JsString> {
        self.available_players
            .iter()
            .map(|player| player.player_id.inner().into())
            .collect()
    }

    #[wasm_bindgen]
    pub fn increase_throw(&mut self) -> JsString {
        self.get_focused().throws += 1;
        self.get_focused().set_throw()
    }

    #[wasm_bindgen]
    pub fn decrease_throw(&mut self) -> JsString {
        self.get_focused().throws -= 1;
        self.get_focused().set_throw()
    }

    #[wasm_bindgen]
    pub fn get_focused_player_names(&self) -> Vec<JsString> {
        vec![
            self.score_card.p1.clone(),
            self.score_card.p2.clone(),
            self.score_card.p3.clone(),
            self.score_card.p4.clone(),
        ]
        .iter()
        .map(|player| player.name.clone().into())
        .collect()
    }
    #[wasm_bindgen(setter)]
    pub fn set_event_id(&mut self, event_id: JsString) {
        self.event_id = String::from(event_id);
    }
}

#[wasm_bindgen]
#[derive(Default, Clone)]
pub struct ScoreCard {
    #[wasm_bindgen(skip)]
    pub p1: get_data::NewPlayer,
    #[wasm_bindgen(skip)]
    pub p2: get_data::NewPlayer,
    #[wasm_bindgen(skip)]
    pub p3: get_data::NewPlayer,
    #[wasm_bindgen(skip)]
    pub p4: get_data::NewPlayer,
    #[wasm_bindgen(skip)]
    pub all_play_players: Vec<get_data::NewPlayer>,
    consts: Constants,
}

#[wasm_bindgen]
impl ScoreCard {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ScoreCard {
        ScoreCard::default()
    }

    #[wasm_bindgen]
    pub fn set_player(
        &mut self,
        player_num: usize,
        player_id: JsString,
        rnd: usize,
    ) -> Vec<JsString> {
        //let player_id = player_id.trim_start_matches("\"").trim_end_matches("\"").to_string();
        let mut out_vec: Vec<JsString> = vec![];
        for player in self.all_play_players.clone() {
            if player.player_id == cynic::Id::from(&player_id) {
                let mut p = player.clone();
                p.ind = player_num - 1;
                out_vec.append(&mut p.clone().set_name());
                out_vec.append(&mut p.clone().set_round(rnd)); // resets score and sets round
                match player_num {
                    1 => self.p1 = p.clone(),
                    2 => self.p2 = p.clone(),
                    3 => self.p3 = p.clone(),
                    4 => self.p4 = p.clone(),
                    _ => (),
                }
            }
        }
        log(&format!("player_id: {}", player_id));
        out_vec
    }

    #[wasm_bindgen]
    pub fn set_total_score(&mut self, player_num: usize, new_score: i16) {
        match player_num {
            1 => self.p1.total_score = new_score,
            2 => self.p2.total_score = new_score,
            3 => self.p3.total_score = new_score,
            4 => self.p4.total_score = new_score,
            _ => panic!("Invalid player number"),
        }
    }

    pub fn set_round(&mut self, round: usize) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];

        return_vec.append(&mut self.p1.set_round(round));
        return_vec.append(&mut self.p2.set_round(round));
        return_vec.append(&mut self.p3.set_round(round));
        return_vec.append(&mut self.p4.set_round(round));
        return_vec
    }
}

#[cfg(test)]
mod tests {
    //! Tests need to run with high node version otherwise it fails!

    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen(module = "src/test_module.js")]
    extern "C" {
        fn sendData(host: &str, port: u16, data: &str);
    }

    async fn generate_app() -> MyApp {
        let mut app = MyApp {
            event_id: "a95092a2-e4ab-4196-a8a6-64de2a1893a8".to_string(),
            ..Default::default()
        };
        app.get_event().await.unwrap();
        log(&format!("{:#?}", app.pools));
        app.set_div(0);
        app.get_players(false);
        let players = app.get_player_ids();
        // app.set_player(1, players[0].clone());
        // app.set_player(2, players[1].clone());
        // app.set_player(3, players[2].clone());
        // app.set_player(4, players[3].clone());
        // app.set_foc(1);
        players.iter().enumerate().take(4 + 1).skip(1).for_each(|(i, player)| {
            let test = app.set_player(i, player.clone());
            log(&format!("{:#?}", test));
            //send(&handle_js_vec(test));
        });
        app.set_foc(0);
        app
    }

    // #[wasm_bindgen_test]
    // async fn test_set_all_to_hole() {
    //     let mut app = generate_app().await;
    //     send(&handle_js_vec(app.set_all_to_hole(13)));
    // }

    fn send(data: &str) {
        sendData("192.168.120.135", 8099, data);
    }
    fn handle_js_vec(js_vec: Vec<JsString>) -> String {
        js_vec
            .iter()
            .map(|s| String::from(s) + "\r\n")
            .collect::<Vec<String>>()
            .join("")
    }
    #[wasm_bindgen_test]
    async fn lb_test() {
        let mut app = generate_app().await;
        let round = 2;
        let thru = 3;

        log("here");

        log("not here");

        app.set_round(round - 1);
        app.set_all_to_hole(thru - 1);


        //send(&handle_js_vec(MyApp::clear_lb(10)));
        let all_commands = handle_js_vec(app.set_leaderboard());
        log(&all_commands);
        send(&all_commands);
        // send(&handle_js_vec(app.show_all_pos()));

        //let return_vec: Vec<JsString> = app.get_focused().start_score_anim();
        //send(&handle_js_vec(return_vec));

        //send(&handle_js_vec(app.ob_anim()));

        // let thingy = MyApp::clear_lb(10).iter()
        //     .map(|s| String::from(s)+"\r\n")
        //     .collect::<Vec<String>>()
        //     .join("");
        // log(&thingy);

        // send(&thingy);

        // log(format!(
        //     "{:#?}",
        //     app.available_players
        //         .iter()
        //         .map(|player| player.name.clone()
        //             + ": "
        //             + &player.round_score.to_string()
        //             + ", "
        //             + &player.total_score.to_string()
        //             + ", "
        //             + &player.position.to_string()
        //             + ", "
        //             + &player.lb_even.to_string()
        //             + ", "
        //             + &player.lb_pos.to_string())
        //         .collect::<Vec<String>>()
        // )
        // .as_str());
    }
}
