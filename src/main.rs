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
#[derive(Default)]
struct MyApp {
    allowed_to_close: bool,
    show_confirmation_dialog: bool,
    counter: i32,
    my_string: String,
    my_f32: f32,
    my_boolean: bool,

}



impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let counter = 0;
        let my_string = "Hello world!".to_string();
        let my_f32: f32 = 0.0;
        let my_boolean = false;
        Self::default()
    }
}

impl eframe::App for MyApp {
    
    fn on_close_event(&mut self) -> bool {
        self.show_confirmation_dialog = true;
        self.allowed_to_close
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Flip UP, Official VMix tool");
            ui.label("This is a label");
            ui.hyperlink("https://github.com/emilk/egui");
            ui.text_edit_singleline(&mut self.my_string);
            if ui.button("Click me").clicked() { }
            ui.add(egui::Slider::new(&mut self.my_f32, 0.0..=100.0));
            ui.add(egui::DragValue::new(&mut self.my_f32));

            ui.checkbox(&mut self.my_boolean, "Checkbox");
            if self.my_boolean {
                ui_counter(ui, &mut self.counter);
            }


            ui.separator();

            
            ui.collapsing("Click to see what is hidden!", |ui| {
                ui.label("Not much, as it turns out");
            });

        });
        
        if self.show_confirmation_dialog {
            // Show confirmation dialog:
            egui::Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_confirmation_dialog = false;
                        }

                        if ui.button("Yes!").clicked() {
                            self.allowed_to_close = true;
                            frame.close();
                        }
                    });
                });
        }
    }
    
}



fn ui_counter(ui: &mut egui::Ui, counter: &mut i32) {
    // Put the buttons and label on the same row:
    ui.horizontal(|ui| {
        if ui.button("-").clicked() {
            *counter -= 1;
        }
        ui.label(counter.to_string());
        if ui.button("+").clicked() {
            *counter += 1;
        }
    });
}