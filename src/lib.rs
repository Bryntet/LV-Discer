mod get_data;
mod utils;
mod vmix;
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
    pool_id: String,
    default_bg_col: String,
    vmix_id: String,
}
impl Default for Constants {
    fn default() -> Self {
        Self {
            ip: "192.168.120.135".to_string(),
            pool_id: "a592cf05-095c-439f-b69c-66511b6ce9c6".to_string(),
            default_bg_col: "3F334D".to_string(),
            vmix_id: "1e8955e9-0925-4b54-9e05-69c1b3bbe5ae".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    div: get_data::queries::PoolLeaderboardDivision,
    selected: usize,
    player: Option<get_data::queries::PoolLeaderboardPlayer>,
    hole: usize,
    num: String,
    consts: Constants,
    throws: u8,
    score: f64,
    shift: usize,
    ob: bool,
    total_score: i32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            div: get_data::queries::PoolLeaderboardDivision {
                id: cynic::Id::from(""),
                name: "".to_owned(),
                players: vec![],
                type_: "".to_owned(),
            },
            selected: 0,
            player: None,
            hole: 0,
            num: "0".to_string(),
            consts: Constants::default(),
            throws: 0,
            score: 0.0,
            shift: 0,
            ob: false,
            total_score: 0,
        }
    }
}

impl Player {
    fn player_selector(&mut self, player: get_data::queries::PoolLeaderboardPlayer) {
        self.player = Some(player);
        self.set_name();
        self.reset_scores();
    }

    fn start_score_anim(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        return_vec.push("FUNCTION OverlayInput4Off".into());
        return_vec.push(self.set_input_pan());
        return_vec.push(self.set_mov_overlay());
        self.ob = false;
        //return_vec.append(&mut self.play_anim());
        return_vec
    }

    fn set_mov_overlay(&mut self) -> JsString {
        format!("FUNCTION OverlayInput4 Input={}", self.get_mov()).into()
    }

    fn set_input_pan(&mut self) -> JsString {
        let pan = match self.num.parse::<u8>().unwrap() {
            1 => -0.628,
            2 => -0.628 + 0.419,
            3 => -0.628 + 0.4185 * 2.0,
            4 => -0.628 + 0.419 * 3.0,
            _ => -0.628,
        };
        format!("FUNCTION SetPanX Value={}&Input={}", pan, self.get_mov()).into()
    }

    fn play_anim(&mut self) -> Vec<JsString> {
        vec![
            format!("FUNCTION Restart Input={}", self.get_mov()).into(),
            format!("FUNCTION Play Input={}", self.get_mov()).into(),
        ]
    }

    fn get_mov(&self) -> String {
        if self.ob {
            "50 ob.mov".to_string()
        } else if let Some(player) = self.player.clone() {
            player.results[self.hole].get_mov().to_string()
        } else {
            "AAA FUCK THIS SHOULDN'T HAPPEN".to_string()
        }
    }

    fn set_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        if let Some(player) = self.player.clone() {
            // self.start_score_anim();
            // wait Xms
            let selection = format!(
                "Input={}&SelectedName={}.Text",
                &self.consts.vmix_id,
                format!("s{}p{}", self.hole + 1 - self.shift, self.num)
            );
            let select_colour = format!(
                "Input={}&SelectedName={}.Fill.Color",
                &self.consts.vmix_id,
                format!("h{}p{}", self.hole + 1 - self.shift, self.num)
            );
            let selection_hole = format!(
                "Input={}&SelectedName={}.Text",
                &self.consts.vmix_id,
                format!("HN{}p{}", self.hole + 1 - self.shift, self.num)
            );
            let result = &player.results[self.hole];

            // Set score
            return_vec
                .push(format!("FUNCTION SetText Value={}&{}", &result.score, &selection).into());
            // Set colour
            return_vec.push(
                format!(
                    "FUNCTION SetColor Value=#{}&{}",
                    &result.get_score_colour(),
                    &select_colour
                )
                .into(),
            );
            // Show score
            return_vec.push(format!("FUNCTION SetTextVisibleOn {}", &selection).into());
            return_vec.push(format!("FUNCTION SetTextVisibleOn {}", &selection_hole).into());
            return_vec.push(
                format!(
                    "FUNCTION SetText Value={}&{}",
                    &self.hole + 1,
                    &selection_hole
                )
                .into(),
            );

            self.score += result.actual_score();
            return_vec.push(self.set_tot_score());
            self.hole += 1;
            self.throws = 0;
            return_vec.push(self.set_throw());
        }
        return_vec
    }

    fn set_tot_score(&mut self) -> JsString {
        let selection = format!(
            "Input={}&SelectedName={}.Text",
            &self.consts.vmix_id,
            format!("scoretotp{}", self.num)
        );
        format!("FUNCTION SetText Value={}&{}", &self.score, &selection).into()
    }

    fn shift_scores(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        let in_hole = self.hole.clone();
        let diff = self.hole - 8;
        self.hole = diff;
        let score = self.score.clone();
        log(&format!("diff: {}", diff));
        for i in diff..in_hole {
            log(&format!("i: {}", i));
            self.shift = diff;
            log(&format!("hole: {}\nshift: {}", self.hole, self.shift));
            return_vec.append(&mut self.set_hole_score());
        }
        self.score = score;
        return_vec.append(&mut self.set_hole_score());
        return_vec
    }

    fn reset_scores(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        for i in 0..9 {
            self.hole = i;
            return_vec.append(&mut self.del_score());
        }
        self.hole = 0;
        self.score = 0.0;
        return_vec.push(self.set_tot_score());
        return_vec
    }

    fn del_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        let selection = format!(
            "Input={}&SelectedName={}.Text",
            &self.consts.vmix_id,
            format!("s{}p{}", self.hole + 1, self.num)
        );
        let select_colour = format!(
            "Input={}&SelectedName={}.Fill.Color",
            &self.consts.vmix_id,
            format!("h{}p{}", self.hole + 1, self.num)
        );
        let selection_hole = format!(
            "Input={}&SelectedName={}.Text",
            &self.consts.vmix_id,
            format!("HN{}p{}", self.hole + 1, self.num)
        );
        return_vec.push(format!("FUNCTION SetText Value={}&{}", "", &selection).into());
        return_vec.push(
            format!(
                "FUNCTION SetColor Value=#{}&{}",
                self.consts.default_bg_col, &select_colour
            )
            .into(),
        );
        return_vec.push(
            format!(
                "FUNCTION SetText Value={}&{}",
                &self.hole + 1,
                &selection_hole
            )
            .into(),
        );
        return_vec.push(format!("FUNCTION SetTextVisibleOff {}", &selection).into());
        return_vec
    }

    fn revert_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        if self.hole > 0 {
            self.hole -= 1;
            return_vec.append(&mut self.del_score());
            log(&format!("{}", self.hole));
            if let Some(player) = &self.player {
                let result = &player.results[self.hole];
                self.score -= result.actual_score();
                if self.hole > 8 {
                    self.hole -= 1;
                    self.score -= &player.results[self.hole].actual_score();
                    return_vec.append(&mut self.shift_scores());
                } else {
                    return_vec.push(self.set_tot_score());
                }
            }
        }
        return_vec
    }

    fn set_name(&mut self) -> JsString {
        if let Some(player) = &self.player {
            let selection = format!(
                "Input={}&SelectedName={}.Text",
                &self.consts.vmix_id,
                format!("namep{}", self.num)
            );
            let name = format!("{} {}", &player.first_name, &player.last_name);
            format!("FUNCTION SetText Value={}&{}", name, &selection).into()
        } else {
            "".into()
        }
    }

    fn set_throw(&self) -> JsString {
        if let Some(_player) = &self.player {
            let selection = format!(
                "Input={}&SelectedName={}.Text",
                &self.consts.vmix_id,
                format!("t#p{}", self.num)
            );

            format!(
                "FUNCTION SetText Value={}&{}",
                self.throws, &selection
            ).into()
        } else {
            "".into()
        }
    }
}
#[wasm_bindgen]
pub struct MyApp {
    id: String,
    name: String,
    text: String,
    #[wasm_bindgen(skip)]
    pub all_divs: Vec<get_data::queries::PoolLeaderboardDivision>,
    #[wasm_bindgen(getter_with_clone)]
    pub score_card: ScoreCard,
    selected_div_ind: usize,
    #[wasm_bindgen(skip)]
    pub selected_div: Option<get_data::queries::PoolLeaderboardDivision>,
    foc_play_ind: usize,
    consts: Constants,
    input_ids: Vec<String>,
    event: Option<get_data::queries::Event>,
    event_id: String,
    pools: Vec<get_data::queries::Pool>,
    pool_ind: usize,
}

impl Default for MyApp {
    fn default() -> MyApp {
        MyApp {
            id: "1e8955e9-0925-4b54-9e05-69c1b3bbe5ae".into(),
            name: "TextBlock3.Text".into(),
            text: "".into(),
            score_card: ScoreCard::default(),
            all_divs: vec![],
            selected_div_ind: 0,
            selected_div: None,
            foc_play_ind: 0,
            consts: Constants::default(),
            input_ids: vec![
                "1e8955e9-0925-4b54-9e05-69c1b3bbe5ae".to_string(),
                "1e8955e9-0925-4b54-9e05-69c1b3bbe5ae".to_string(),
                "1e8955e9-0925-4b54-9e05-69c1b3bbe5ae".to_string(),
                "1e8955e9-0925-4b54-9e05-69c1b3bbe5ae".to_string(),
            ],
            event: None,
            pools: vec![],
            pool_ind: 0,
            event_id: "a57b4ed6-f64a-4710-8f20-f93e82d4fe79".into(),
        }
    }
}

#[wasm_bindgen]
impl MyApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> MyApp {
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
        self.selected_div = Some(self.all_divs[idx].clone());
        self.score_card.all_play_players = self.selected_div.as_ref().unwrap().players.clone();
    }

    #[wasm_bindgen(setter = round)]
    pub fn set_round(&mut self, idx: usize) {
        self.pool_ind = idx;
    }

    #[wasm_bindgen]
    pub async fn get_event(&mut self) -> Result<JsValue, JsValue> {
        self.pools = vec![];
        let mut promise: usize = 0;
        if let Some(data) = get_data::post_status(cynic::Id::from(&self.event_id))
            .await
            .data
        {
            if let Some(event) = data.event {
                for round in &event.rounds {
                    if let Some(round) = round {
                        for pool in &round.pools {
                            self.pools.push(pool.clone());
                        }
                    }
                }
                promise = event.rounds.len();
                self.event = Some(event);
            }
        }
        let promise = js_sys::Promise::resolve(&JsValue::from_str(&promise.to_string()));
        let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(result)
    }
    
    #[wasm_bindgen(getter)]
    pub fn rounds(&self) -> usize {
        self.pools.len()+1
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
        log("increase_score");
        let hole = self.get_focused().hole;
        log(format!("hole: {}", hole).as_str());
        if hole <= 17 {
            if hole <= 8 {
                self.get_focused().set_hole_score()
            } else {
                self.get_focused().shift_scores()
            }
        } else {
            vec![]
        }
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
        let mut t = self.get_focused().start_score_anim();
        t.push(self.increase_throw());
        t
    }

    fn get_focused(&mut self) -> &mut Player {
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
        self.score_card.set_player(idx, player)
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
        self.get_focused().reset_scores()
    }

    #[wasm_bindgen]
    pub fn get_foc_p_name(&mut self) -> JsString {
        if let Some(player) = self.get_focused().player.clone() {
            format!("{} {}", player.first_name, player.last_name).into()
        } else {
            "".into()
        }
    }

    #[wasm_bindgen]
    pub fn get_div_names(&self) -> Vec<JsString> {
        let mut return_vec = vec![];
        for div in &self.all_divs {
            return_vec.push(div.name.clone().into());
        }
        return_vec
    }

    #[wasm_bindgen]
    pub fn get_player_names(&self) -> Vec<JsString> {
        let mut return_vec = vec![];
        if let Some(div) = &self.selected_div {
            for player in &div.players {
                return_vec.push(format!(
                    "{} {}",
                    &player.first_name, &player.last_name
                ).into());
            }
        }
        return_vec
    }

    #[wasm_bindgen]
    pub fn get_player_ids(&self) -> Vec<JsString> {
        let mut return_vec = vec![];
        if let Some(div) = &self.selected_div {
            for player in &div.players {
                return_vec.push(JsString::from(
                    format!("{:?}", player.player_id)
                        .trim_start_matches("Id(\"")
                        .trim_end_matches("\")")
                        .to_string(),
                ));
            }
        }
        return_vec
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
        let mut return_vec = vec![];
        for player in vec![
            self.score_card.p1.clone(),
            self.score_card.p2.clone(),
            self.score_card.p3.clone(),
            self.score_card.p4.clone(),
        ] {
            if let Some(player) = player.player {
                return_vec.push(JsString::from(format!(
                    "{} {}",
                    &player.first_name, &player.last_name
                )));
            } else {
                return_vec.push(JsString::from(""));
            }
        }
        return_vec
    }
    #[wasm_bindgen(setter)]
    pub fn set_event_id(&mut self, event_id: JsString) {
        self.event_id = String::from(event_id);
    }
    #[wasm_bindgen(setter)]
    pub fn set_pool_id(&mut self, pool_id: JsString) {
        self.consts.pool_id = String::from(pool_id);
    }
}

#[wasm_bindgen]
#[derive(Default, Clone)]
pub struct ScoreCard {
    #[wasm_bindgen(skip)]
    pub p1: Player,
    #[wasm_bindgen(skip)]
    pub p2: Player,
    #[wasm_bindgen(skip)]
    pub p3: Player,
    #[wasm_bindgen(skip)]
    pub p4: Player,
    #[wasm_bindgen(skip)]
    pub all_players: Vec<Player>,
    #[wasm_bindgen(skip)]
    pub all_play_players: Vec<get_data::queries::PoolLeaderboardPlayer>,
    consts: Constants,
}

#[wasm_bindgen]
impl ScoreCard {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ScoreCard {
        ScoreCard::default()
    }

    #[wasm_bindgen]
    pub fn set_player(&mut self, player_num: usize, player_id: JsString) -> Vec<JsString> {
        //let player_id = player_id.trim_start_matches("\"").trim_end_matches("\"").to_string();
        let mut out_vec = vec![];
        for player in &self.all_play_players {
            if player.player_id == cynic::Id::from(&player_id) {
                let mut new_player = Player {
                    player: Some(player.clone()),
                    num: (player_num).to_string(),
                    consts: self.consts.clone(),
                    ..Default::default()
                };
                out_vec.push(new_player.set_name());
                out_vec.append(&mut new_player.reset_scores());
                match player_num {
                    1 => self.p1 = new_player,
                    2 => self.p2 = new_player,
                    3 => self.p3 = new_player,
                    4 => self.p4 = new_player,
                    _ => (),
                }
            }
        }
        out_vec
    }
    
    #[wasm_bindgen]
    pub fn set_total_score(&mut self, player_num: usize, new_score: i32) {
        match player_num {
            1 => self.p1.total_score = new_score,
            2 => self.p2.total_score = new_score,
            3 => self.p3.total_score = new_score,
            4 => self.p4.total_score = new_score,
            _ => panic!("Invalid player number"),
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests need to run with high node version otherwise it fails! 
    use super::*;
    use tokio_test;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::*;

    async fn generate_app() -> MyApp {
        println!("hi");
        let mut app = MyApp::default();

        
        let _ = app.get_event().await.unwrap();

        
        println!("{}", app.pools.len());

        

        //app.set_div(0);
        // let ids = app.get_player_ids();
        // for i in 1..5 {
        //     app.set_player(i.clone(), ids[i.clone()].clone().into());
        // }
        // app.set_foc(1);
        app
    }

    #[wasm_bindgen_test]
    async fn get_rounds() {
        let mut app = generate_app().await;

        let mut ind = 0;
        for pool in &app.pools {
            log(&ind.to_string());
            ind += 1;
            if let Some(lb) = pool.leaderboard.clone() {
                for thing in lb {
                    if let Some(div) = thing {
                        
                        match div {
                            get_data::queries::PoolLeaderboardDivisionCombined::PLD(d) => {
                                log(&d.name);
                                
                            }
                            _ => {}
                        }
                    }
                }
            }   
        }
    }
}
