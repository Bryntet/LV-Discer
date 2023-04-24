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
            ip: "37.123.135.170".to_string(),
            pool_id: "a592cf05-095c-439f-b69c-66511b6ce9c6".to_string(),
            default_bg_col: "3F334D".to_string(),
            vmix_id: "506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),
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
        return_vec.push(self.set_mov_overlay());
        return_vec.push(self.set_input_pan());
        return_vec.push(self.play_anim());
        return_vec
    }
    
    fn set_mov_overlay(&mut self) -> JsString {
        format!("http://{}:8088/api/?Function=OverlayInput4&Input={}", self.consts.ip, self.get_mov()).into()
    }

    fn set_input_pan(&mut self) -> JsString {
        let pan = match self.num.parse::<u8>().unwrap() {
            1 => -0.628,
            2 => -0.628+0.419,
            3 => -0.628+0.419*2.0,
            4 => -0.628+0.419*3.0,
            _ => -0.628,
        };
        format!("http://{}:8088/api/?Function=SetPanX&Value={}&Input={}", self.consts.ip, pan, self.get_mov()).into()
    }
    
    fn play_anim(&mut self) -> JsString {
        format!("http://{}:8088/api/?Function=Play&Input={}", self.consts.ip, self.get_mov()).into()
    }
    
    fn get_mov(&self) -> String {
        let mut mov = "0".to_string();
        
        if let Some(player) = self.player.clone() {
            mov = player.results[self.hole].get_mov().to_string();
        }
        mov
    }

    fn set_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec: Vec<JsString> = vec![];
        if let Some(player) = self.player.clone() {
            // self.start_score_anim();
            // wait Xms
            let selection = format!(
                "&Input={}&SelectedName={}.Text",
                &self.consts.vmix_id,
                format!("s{}p{}", self.hole + 1, self.num)
            );
            let select_colour = format!(
                "&Input={}&SelectedName={}.Fill.Color",
                &self.consts.vmix_id,
                format!("h{}p{}", self.hole + 1, self.num)
            );
            let url = format!("http://{}:8088/api/?", self.consts.ip);
            let result = &player.results[self.hole];
            

            // Set score
            return_vec.push(
                format!(
                    "{}Function=SetText&Value={}{}",
                    &url, &result.score, &selection
                )
                .into(),
            );
            // Set colour
            return_vec.push(
                format!(
                    "{}Function=SetColor&Value=%23{}{}",
                    &url,
                    &result.get_score_colour(),
                    &select_colour
                )
                .into(),
            );
            // Show score
            return_vec.push(format!("{}Function=SetTextVisibleOn{}", &url, &selection).into());

            self.score += result.actual_score();
            return_vec.push(self.set_tot_score());
            self.hole += 1;
            self.throws = 0;
            return_vec.push(self.set_throw());
            return return_vec;
        }
        return vec![];
    }

    fn set_tot_score(&mut self) -> JsString {
        let selection = format!(
            "&Input={}&SelectedName={}.Text",
            &self.consts.vmix_id,
            format!("scoretotp{}", self.num)
        );
        let url = format!("http://{}:8088/api/?", self.consts.ip);
        format!(
            "{}Function=SetText&Value={}{}",
            &url, &self.score, &selection
        )
        .into()
    }

    fn reset_scores(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        for i in 1..19 {
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
        let url = format!("http://{}:8088/api/?", self.consts.ip);
        let selection = format!(
            "&Input={}&SelectedName={}.Text",
            &self.consts.vmix_id,
            format!("s{}p{}", self.hole, self.num)
        );
        let select_colour = format!(
            "&Input={}&SelectedName={}.Fill.Color",
            &self.consts.vmix_id,
            format!("h{}p{}", self.hole, self.num)
        );
        return_vec.push(format!("{}Function=SetText&Value={}{}", &url, "", &selection).into());
        return_vec.push(
            format!(
                "{}Function=SetColor&Value=%23{}{}",
                &url, self.consts.default_bg_col, &select_colour
            )
            .into(),
        );
        return_vec.push(format!("{}Function=SetTextVisibleOff{}", &url, &selection).into());
        return_vec
    }

    fn revert_hole_score(&mut self) -> Vec<JsString> {
        let mut return_vec = vec![];
        if self.hole > 0 {
            return_vec.append(&mut self.del_score());
            self.hole -= 1;
            log(&format!("{}", self.hole));
            if let Some(player) = &self.player {
                let result = &player.results[self.hole];
                self.score -= result.actual_score();
                return_vec.push(self.set_tot_score());
            }
        }
        return_vec
    }

    fn set_name(&mut self) -> JsString {
        if let Some(player) = &self.player {
            let url = format!("http://{}:8088/api/?", self.consts.ip);
            let selection = format!(
                "&Input={}&SelectedName={}.Text",
                &self.consts.vmix_id,
                format!("namep{}", self.num)
            );
            let name = format!("{} {}", &player.first_name, &player.last_name);
            JsString::from(format!(
                "{}Function=SetText&Value={}{}",
                &url, name, &selection
            ))
        } else {
            JsString::from("")
        }
    }

    fn set_throw(&self) -> JsString {
        if let Some(player) = &self.player {
            let url = format!("http://{}:8088/api/?", self.consts.ip);
            let selection = format!(
                "&Input={}&SelectedName={}.Text",
                &self.consts.vmix_id,
                format!("t%23p{}", self.num)
            );
             
            JsString::from(format!(
                "{}Function=SetText&Value={}{}",
                &url, self.throws, &selection
            ))
        } else {
            JsString::from("")
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
}

impl Default for MyApp {
    fn default() -> MyApp {
        MyApp {
            id: String::from("506fbd14-52fc-495b-8d17-5b924fba64f3"),
            name: String::from("TextBlock3.Text"),
            text: String::from(""),
            score_card: ScoreCard::default(),
            all_divs: vec![],
            selected_div_ind: 0,
            selected_div: None,
            foc_play_ind: 0,
            consts: Constants::default(),
            input_ids: vec![
                "506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),
                "506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),
                "506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),
                "506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),
            ],
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
        self.consts.ip = ip;
    }
    #[wasm_bindgen(setter = div)]
    pub fn set_div(&mut self, idx: usize) {
        self.selected_div = Some(self.all_divs[idx].clone());
        self.score_card.all_play_players = self.selected_div.as_ref().unwrap().players.clone();
    }

    #[wasm_bindgen]
    pub async fn get_divs(&mut self) -> Result<JsValue, JsValue> {
        self.all_divs = vec![];
        let mut res = String::new();
        if let Err(e) = get_data::request_tjing(cynic::Id::from(&self.consts.pool_id)).await {
            res = e.to_string()
        }
        if let Some(data) = get_data::post_status(cynic::Id::from(&self.consts.pool_id))
            .await
            .data
        {
            if let Some(pool) = data.pool {
                if let Some(lb) = pool.leaderboard {
                    for div in lb {
                        if let Some(div) = div {
                            match div {
                                get_data::queries::PoolLeaderboardDivisionCombined::PoolLeaderboardDivision(d) => {
                                    self.all_divs.push(d);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            res = format!("{:#?}", self.all_divs);
        }
        let promise = js_sys::Promise::resolve(&res.into());
        let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(result)
    }

    #[wasm_bindgen]
    pub fn increase_score(&mut self) -> Vec<JsString> {
        log("increase_score");
        self.get_focused().set_hole_score()
    }

    #[wasm_bindgen]
    pub fn play_animation(&mut self) -> Vec<JsString> {
        log("play_animation");
        self.get_focused().start_score_anim()
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
            return_vec.push(JsString::from(div.name.clone()));
        }
        return_vec
    }

    #[wasm_bindgen]
    pub fn get_player_names(&self) -> Vec<JsString> {
        let mut return_vec = vec![];
        if let Some(div) = &self.selected_div {
            for player in &div.players {
                return_vec.push(JsString::from(format!(
                    "{} {}",
                    &player.first_name, &player.last_name
                )));
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
        for player in vec![self.score_card.p1.clone(), self.score_card.p2.clone(), self.score_card.p3.clone(), self.score_card.p4.clone()] {
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
    test: String,
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
            log(&format!(
                "{:?}, {:?}",
                &player.player_id,
                cynic::Id::from(&player_id)
            ));
            if player.player_id == cynic::Id::from(&player_id) {
                let mut new_player = Player {
                    player: Some(player.clone()),
                    num: (player_num).to_string(),
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
}
