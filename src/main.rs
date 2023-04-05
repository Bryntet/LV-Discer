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
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let counter = 0;
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
            ui.heading("Try to close the window");
            ui_counter(ui, &mut self.counter);
        });

        if self.show_confirmation_dialog {
            // Show confirmation dialog:
            egui::Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("This is a label");
                    ui.hyperlink("https://github.com/emilk/egui");
                    ui.text_edit_singleline(&mut my_string);
                    if ui.button("Click me").clicked() { }
                    ui.add(egui::Slider::new(&mut my_f32, 0.0..=100.0));
                    ui.add(egui::DragValue::new(&mut my_f32));

                    ui.checkbox(&mut my_boolean, "Checkbox");

                    #[derive(PartialEq)]
                    enum Enum { First, Second, Third }
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut my_enum, Enum::First, "First");
                        ui.radio_value(&mut my_enum, Enum::Second, "Second");
                        ui.radio_value(&mut my_enum, Enum::Third, "Third");
                    });

                    ui.separator();

                    ui.image(my_image, [640.0, 480.0]);

                    ui.collapsing("Click to see what is hidden!", |ui| {
                        ui.label("Not much, as it turns out");
                    });


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