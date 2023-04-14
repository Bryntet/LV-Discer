mod get_data;
mod utils;
mod vmix;
use wasm_bindgen::prelude::*;
use js_sys::JsString;

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
#[derive(Clone, Debug)]
struct Constants {
    ip: String,
    pool_id: String,
    default_bg_col: String,
}
impl Default for Constants {
    fn default() -> Self {
        Self {
            ip: "37.123.135.170".to_string(),
            pool_id: "a592cf05-095c-439f-b69c-66511b6ce9c6".to_string(),
            default_bg_col: "3F334D".to_string(),
        }
    }
}


#[derive(Clone)]
pub struct Player {
    div: get_data::queries::PoolLeaderboardDivision,
    selected: usize,
    player: Option<get_data::queries::PoolLeaderboardPlayer>,
    hole: usize,
    input_id: String,
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
            input_id: "".to_owned(),
            num: "0".to_string(),
            consts: Constants::default(),
            throws: 0,
            score: 0.0,
        }
    }
}

impl Player {
    pub fn player_selector(&mut self, player: get_data::queries::PoolLeaderboardPlayer) {
        self.player = Some(player);
        self.set_name();
        self.reset_scores();
    }

    fn start_score_anim(&mut self) {
        // Code for anim goes here
    }

    pub fn set_hole_score(&mut self) -> Vec<String> {
        println!("{}", self.hole);
        let mut return_vec = vec![];
        if let Some(player) = self.player.clone() {
            self.start_score_anim();
            // wait Xms
            let selection = format!(
                "&Input={}&SelectedName={}.Text",
                &self.input_id,
                format!("s{}p{}", self.hole + 1, self.num)
            );
            let select_colour = format!(
                "&Input={}&SelectedName={}.Fill.Color",
                &self.input_id,
                format!("h{}p{}", self.hole + 1, self.num)
            );
            let url = format!("http://{}:8088/api/?", self.consts.ip);
            let result = &player.results[self.hole];
            println!(
                "{}",
                format!(
                    "{}Function=SetColor&Value=%23{}{}",
                    &url,
                    &result.get_score_colour(),
                    &select_colour
                )
            );

            // Set score
            return_vec.push(format!(
                "{}Function=SetText&Value={}{}",
                &url, &result.score, &selection
            ));
            // Set colour
            return_vec.push(format!(
                "{}Function=SetColor&Value=%23{}{}",
                &url,
                &result.get_score_colour(),
                &select_colour
            ));
            // Show score
            return_vec.push(format!("{}Function=SetTextVisibleOn{}", &url, &selection));

            self.score += result.score;
            return_vec.push(self.set_tot_score());
            self.hole += 1;
        }
        println!("{}", self.hole);
        return_vec
    }

    fn set_tot_score(&mut self) -> String {
        let selection = format!(
            "&Input={}&SelectedName={}.Text",
            &self.input_id,
            format!("scoretotp{}", self.num)
        );
        let url = format!("http://{}:8088/api/?", self.consts.ip);
        format!(
            "{}Function=SetText&Value={}{}",
            &url, &self.score, &selection
        )
    }

    fn reset_scores(&mut self) -> Vec<String> {
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

    fn del_score(&mut self) -> Vec<String> {
        let mut return_vec = vec![];
        let url = format!("http://{}:8088/api/?", self.consts.ip);
        let selection = format!(
            "&Input={}&SelectedName={}.Text",
            &self.input_id,
            format!("s{}p{}", self.hole, self.num)
        );
        let select_colour = format!(
            "&Input={}&SelectedName={}.Fill.Color",
            &self.input_id,
            format!("h{}p{}", self.hole, self.num)
        );
        return_vec.push(format!(
            "{}Function=SetText&Value={}{}",
            &url, "", &selection
        ));
        return_vec.push(format!(
            "{}Function=SetColor&Value=%23{}{}",
            &url, self.consts.default_bg_col, &select_colour
        ));
        return_vec.push(format!("{}Function=SetTextVisibleOff{}", &url, &selection));
        return_vec
    }

    pub fn revert_hole_score(&mut self) -> Vec<String> {
        let mut return_vec = vec![];
        if self.hole > 0 {
            return_vec.append(&mut self.del_score());
            self.hole -= 1;
            if let Some(player) = &self.player {
                let result = &player.results[self.hole];
                self.score -= result.score;
                return_vec.push(self.set_tot_score());
            }
        }
        return_vec
    }

    pub fn set_name(&mut self) -> String {
        if let Some(player) = &self.player {
            let url = format!("http://{}:8088/api/?", self.consts.ip);
            let selection = format!(
                "&Input={}&SelectedName={}.Text",
                &self.input_id,
                format!("namep{}", self.num)
            );
            let name = format!("{} {}", &player.first_name, &player.last_name);
            String::from(format!(
                "{}Function=SetText&Value={}{}",
                &url, name, &selection
            ))
        } else {
            String::from("")
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
    #[wasm_bindgen(skip)]
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
    #[wasm_bindgen(setter = div)]
    pub fn set_div(&mut self, div_name: String) {
        for (i, div) in self.all_divs.iter().enumerate() {
            if div.name == div_name {
                self.selected_div_ind = i;
                self.selected_div = Some(div.clone());
                break;
            }
        }
    }
    #[wasm_bindgen]
    pub async fn get_divs(&mut self) -> Result<JsValue, JsValue> {
        log(&format!("{:#?}", &self.consts));
        let mut res = String::new(); 
        if let Err(e) = get_data::request_tjing(cynic::Id::from(&self.consts.pool_id)).await {
            log(&format!("Error:{:#?}", e));
            res = e.to_string()
        }
        if let Some(data) = get_data::post_status(cynic::Id::from(&self.consts.pool_id)).await.data {
            log(&format!("Data:{:#?}", data));
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
        log(&res);
        let promise = js_sys::Promise::resolve(&res.into());
        let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(result)
    }
    
    #[wasm_bindgen]
    pub fn get_div_names(&self) -> Vec<JsString> {
        let mut return_vec = vec![];
        for div in &self.all_divs {
            return_vec.push(JsString::from(div.name.clone()));
        }
        return_vec
    }
    
    #[wasm_bindgen(setter)]
    pub fn set_ip(&mut self, ip: JsString) {
        self.consts.ip = String::from(ip);
    }
    #[wasm_bindgen(setter)]
    pub fn set_pool_id(&mut self, pool_id: JsString) {
        log("setting pool id");
        log(&format!("{:#?}",pool_id));
        self.consts.pool_id = String::from(pool_id);
    }
}

#[wasm_bindgen]
#[derive(Default)]
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
}

#[wasm_bindgen]
impl ScoreCard {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ScoreCard {
        ScoreCard::default()
    }

    #[wasm_bindgen]
    pub fn set_player(&mut self, player_num: u8, player_id: String) {
        log(&format!("{} {}", player_num, player_id));
        
        for player_ in &self.all_players {
            if let Some(player) = &player_.player{
                if player.player_id == cynic::Id::from(&player_id) {
                    match player_num {
                        1 => self.p1 = player_.clone(),
                        2 => self.p2 = player_.clone(),
                        3 => self.p3 = player_.clone(),
                        4 => self.p4 = player_.clone(),
                        _ => (),
                    }
                    break;
                }
            }
        }
    }
}
