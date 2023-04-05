mod vmix;
use eframe::egui;


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Confirm exit",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

struct MyApp {
    allowed_to_close: bool,
    show_confirmation_dialog: bool,
    ip: String,
    id: String,
    name: String,
    text: String,
    box_iteration: u8,
    box_color: egui::Color32,

}


impl Default for MyApp {
    fn default() -> MyApp {
        MyApp {
            allowed_to_close: false,
            show_confirmation_dialog: false,
            ip: String::from("192.168.120.109"),
            id: String::from("909fecdd-3c51-4308-9a37-5365a1eb261c"),
            name: String::from("TextBlock3.Text"),
            text: String::from(""),
            box_iteration: 1,
            box_color: egui::Color32::from_rgb(255, 0, 0),
        }
    }
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
                    ui.text_edit_singleline(&mut self.ip);
                });

                ui.color_edit_button_srgba(&mut self.box_color);
            });
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
                    ip: self.ip.clone()
                };
            text.name_format(self.box_iteration);

            if ui.button("Update text").clicked() { 
                text.set_text(&self.text);
            }
            ui.separator();

            if ui.button("Toggle visibility").clicked() { 
                text.toggle_visibility();
            }
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