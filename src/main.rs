mod vmix;
mod get_data;
use eframe::egui;
use hyper::http::request;


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Flip UP -- Official VMix tool",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

#[derive(Clone)]
struct Constants {
    ip: String,
    pool_id: String,
    
}
impl Default for Constants {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            pool_id: "a592cf05-095c-439f-b69c-66511b6ce9c6".to_string()
        }
    }
}
#[derive(Clone)]
struct Player {
    div: get_data::queries::PoolLeaderboardDivision,
    selected: usize,
    player: Option<get_data::queries::PoolLeaderboardPlayer>,
    hole: usize,
    input_id: String,
    response: Option<egui::Response>,
    num: String,
    consts: Constants,
}

impl Player {
    fn player_selector(&mut self, ui: &mut egui::Ui, div: &get_data::queries::PoolLeaderboardDivision, ui_id: &str) {
        self.num = ui_id.to_string();
        let player_list = div.players.iter().map(|p| p.first_name.to_owned()).collect::<Vec<String>>();
        ui.push_id(ui_id, |ui| {
            ui.vertical(|ui| {
                ui.label(format!("Player {}", ui_id));
                if egui::ComboBox::from_label("").show_index(
                    ui,
                    &mut self.selected,
                    div.players.len(),
                    |i| player_list[i].to_owned()
                ).changed() {
                    if let Some(player) = div.players.get(self.selected) {
                        self.player = Some(player.clone());
                    }
                }
            });
        });
    }

    fn start_score_anim(&mut self) {
        // Code for anim goes here
    }

    

    fn set_score(&mut self, ui: &mut egui::Ui) {
        if let Some(player) = self.player.clone() {
            ui.vertical(|ui| {
                if ui.button("Set score {}").clicked() {
                    self.start_score_anim();
                    // wait Xms
                    let name = format!("{}.Text", self.input_id);
                    let selection = format!("&Input={}&SelectedName={}.Text", &self.input_id, format!("s{}p{}",self.hole+1,self.num));
                    let select_colour = format!("&Input={}&SelectedName={}.Fill.Color", &self.input_id, format!("h{}p{}",self.hole+1,self.num));
                    let url = format!("http://{}:8088/api/?",self.consts.ip);
                    let result = &player.results[self.hole];
                    println!("{}Function=SetColor&Value={}{}", &url, &result.get_score_colour(), &select_colour);
                    // Set score
                    reqwest::blocking::get(format!("{}Function=SetText&Value={}{}", &url, &result.score, &selection)).unwrap();
                    // Set colour
                    reqwest::blocking::get(format!("{}Function=SetColor&Value=%23{}{}", &url, &result.get_score_colour(), &select_colour)).unwrap();
                    // Show score
                    reqwest::blocking::get(format!("{}Function=SetTextVisibleOn{}", &url, &selection)).unwrap();
                    self.hole += 1;
                }
            });
        }
    }
    
    
    
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
            response: None,
            num: "0".to_string(),
            consts: Constants::default(),
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
enum MyEnum { First, Second, Third}
struct MyApp {
    allowed_to_close: bool,
    show_confirmation_dialog: bool,
    id: String,
    name: String,
    text: String,
    box_iteration: u8,
    box_color: egui::Color32,
    selected: usize,
    all_divs: Vec<get_data::queries::PoolLeaderboardDivision>,
    score_card: ScoreCard,
    selected_div_ind: usize,
    selected_div: Option<get_data::queries::PoolLeaderboardDivision>,
    focused_player: Option<Player>,
    foc_play_ind: usize,
    consts: Constants,
    input_ids: Vec<String>,

}
impl Default for MyApp {
    fn default() -> MyApp {
        MyApp {
            allowed_to_close: false,
            show_confirmation_dialog: false,
            id: String::from("506fbd14-52fc-495b-8d17-5b924fba64f3"),
            name: String::from("TextBlock3.Text"),
            text: String::from(""),
            box_iteration: 1,
            box_color: egui::Color32::from_rgb(255, 0, 0),
            selected: 0,
            score_card: ScoreCard::default(),
            all_divs: vec![], 
            selected_div_ind: 0,
            selected_div: None,
            focused_player: None,
            foc_play_ind: 0,
            consts: Constants::default(),
            input_ids: vec!["506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),"506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),"506fbd14-52fc-495b-8d17-5b924fba64f3".to_string(),"506fbd14-52fc-495b-8d17-5b924fba64f3".to_string()]
        }
    }
    
}


impl MyApp {
    async fn get_all_divs(&mut self) {

        self.all_divs = vec![];
        let response = get_data::request_tjing(cynic::Id::from(self.consts.pool_id.clone())).await.unwrap();
        let mut cont = false;
        if let Some(err) = response.errors {
            println!("Got error, probably invalid pool id\n{:?}", err);
        } else if let Some(data) = response.data {
            if let Some(pool) = data.pool {
                match pool.status {
                    get_data::queries::PoolStatus::Completed => cont = true,
                    _ => println!("Status is not completed")
                }
            }
        }
        
        

        if cont {
            let response = get_data::post_status(cynic::Id::from(self.consts.pool_id.clone())).await.unwrap();
            if let Some(err) = response.errors {
                println!("Got error, probably invalid pool id\n{:?}", err);
            } else if let Some(data) = response.data {
                if let Some(pool) = data.pool {
                    if let Some(lb_divs) = pool.leaderboard {
                        for div in lb_divs {
                            if let Some(div) = div {
                                match div {
                                    get_data::queries::PoolLeaderboardDivisionCombined::PoolLeaderboardDivision(div) => {
                                        self.all_divs.push(div);
                                    },
                                    get_data::queries::PoolLeaderboardDivisionCombined::Unknown => {
                                        println!("Unknown division: {:?}", div);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn choose_div(&mut self, ui: &mut egui::Ui) -> bool {
        return egui::ComboBox::from_label("Choose div").show_index(
            ui, 
            &mut self.selected_div_ind, 
            self.all_divs.len(), 
            |i| self.all_divs[i].name.to_owned()
        ).changed();
    }

    fn add_players(&mut self, ui: &mut egui::Ui) {
        if let Some(div) = &self.selected_div {
            self.score_card.p1.player_selector(ui, &div, "1");
            if let Some(_) = self.score_card.p1.player.clone() {
                self.score_card.p2.player_selector(ui, &div, "2");    
            }
            if let Some(_) = self.score_card.p2.player.clone() {
                self.score_card.p3.player_selector(ui, &div, "3");
            }
            if let Some(_) = self.score_card.p3.player.clone() {
                self.score_card.p4.player_selector(ui, &div, "4");
            }
        }   
    }

    fn player_focus(&mut self, ui: &mut egui::Ui) {
        if let Some(player) = &self.focused_player {
            ui.heading(player.player.as_ref().unwrap().first_name.to_owned());
        } else {
            ui.heading("No player selected");
        }
        ui.horizontal(|ui| {
            if ui.button("-").clicked() && self.foc_play_ind > 1{
                self.foc_play_ind -= 1;
            }
            ui.label(self.foc_play_ind.to_string());
            if ui.button("+").clicked() && self.foc_play_ind < 4 {
                self.foc_play_ind += 1;
            }
            self.focused_player = match self.foc_play_ind {
                1 => Some(self.score_card.p1.clone()),
                2 => Some(self.score_card.p2.clone()),
                3 => Some(self.score_card.p3.clone()),
                4 => Some(self.score_card.p4.clone()),
                _ => None
            };
            
        });
        
    }
}
#[derive(Default)]
struct ScoreCard {
    p1: Player,
    p2: Player,
    p3: Player,
    p4: Player
}



impl eframe::App for MyApp {
    
    // fn on_close_event(&mut self) -> bool {
    //     self.show_confirmation_dialog = true;
    //     self.allowed_to_close
    // }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            
            catppuccin_egui::set_theme(&ctx, catppuccin_egui::MACCHIATO);
            ui.heading("Flip UP -- Official VMix tool");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("ID");
                    ui.text_edit_singleline(&mut self.id);
                });
                ui.vertical(|ui| {
                    ui.label("IP");
                    ui.text_edit_singleline(&mut self.consts.ip);
                });
                ui.vertical(|ui| {
                    ui.label("Round ID");
                    ui.text_edit_singleline(&mut self.consts.pool_id);
                });
                ui.color_edit_button_srgba(&mut self.box_color);
            });
            self.score_card.p1.consts = self.consts.clone();
            self.score_card.p1.input_id = self.input_ids[0].clone();
            self.score_card.p2.consts = self.consts.clone();
            self.score_card.p2.input_id = self.input_ids[1].clone();
            self.score_card.p3.consts = self.consts.clone();
            self.score_card.p3.input_id = self.input_ids[2].clone();
            self.score_card.p4.consts = self.consts.clone();
            self.score_card.p4.input_id = self.input_ids[3].clone();
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Box iteration");
                ui_counter(ui, &mut self.box_iteration);
            });
            ui.horizontal(|ui| {
                ui.label("Text");
                ui.text_edit_singleline(&mut self.text);
            });
            let mut text = vmix::Text {
                    id: self.id.clone(),
                    name: self.name.clone(),
                    text: self.text.clone(),
                    ip: self.consts.ip.clone()
                };

            
            text.name_format(self.box_iteration, "s");

            if ui.button("Update text").clicked() { 
                text.set_text(&self.text);
            }
            ui.separator();
            if ui.button("Toggle visibility").clicked() { 
                text.toggle_visibility();
            }
            ui.separator();
           
            ui.horizontal(|ui| {
                if ui.button("Get event").clicked() { 
                    use async_io::block_on;
                    block_on(self.get_all_divs());
                }
                if self.all_divs.len() > 0 {
                    if self.choose_div(ui) {
                        self.selected_div = Some(self.all_divs[self.selected_div_ind].clone());
                        self.score_card.p1.div = self.all_divs[self.selected_div_ind].clone();
                        self.score_card.p2.div = self.all_divs[self.selected_div_ind].clone();
                        self.score_card.p3.div = self.all_divs[self.selected_div_ind].clone();
                        self.score_card.p4.div = self.all_divs[self.selected_div_ind].clone();
                    }
                }
            });
            

            
            ui.horizontal(|ui| {
                self.add_players(ui);
            });
            ui.separator();
            if let Some(_) = self.score_card.p4.player {
                self.player_focus(ui);
            }
            if let Some(player) = &mut self.focused_player {
                player.set_score(ui);
            }
            ui.separator();
           
        });
        
        // if self.show_confirmation_dialog {
        //      Show confirmation dialog:
        //     egui::Window::new("Do you want to quit?")
        //         .collapsible(false)
        //         .resizable(false)
        //         .show(ctx, |ui| {
                    

        //             ui.horizontal(|ui| {
        //                 if ui.button("Cancel").clicked() {
        //                     self.show_confirmation_dialog = false;
        //                 }

        //                 if ui.button("Yes!").clicked() {
        //                     self.allowed_to_close = true;
        //                     frame.close();
        //                 }
        //             });
        //         });
        // }
    }
    
}



fn ui_counter(ui: &mut egui::Ui, counter: &mut u8) {
    // Put the buttons and label on the same row:
    ui.horizontal(|ui| {
        if ui.button("-").clicked() && *counter > 1{
            *counter -= 1;
        }
        ui.label(counter.to_string());
        if ui.button("+").clicked() {
            *counter += 1;
        }
    });
}