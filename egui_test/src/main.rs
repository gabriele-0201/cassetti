use eframe::egui;

#[derive(PartialEq, Debug)]
enum Enum {
    First,
    Second,
    Third,
}

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    //tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

struct MyApp {
    freq: u32,
    e: Enum,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            freq: 100,
            e: Enum::Second,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            /*
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            */
            ui.add(egui::Slider::new(&mut self.freq, 0..=100).text("age"));
            /*
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
            */

            // PLOT
            use egui::plot::{Line, Plot, PlotPoints};
            let sin: PlotPoints = (0..1000)
                .map(|i| {
                    let x = i as f64 * 0.01;
                    [x, (x * self.freq as f64).sin()]
                })
                .collect();
            let line = Line::new(sin);
            Plot::new("my_plot")
                .view_aspect(2.0)
                .show(ui, |plot_ui| plot_ui.line(line));

            egui::ComboBox::from_label("Select one!")
                .selected_text(format!("{:?}", self.e))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.e, Enum::First, "First");
                    ui.selectable_value(&mut self.e, Enum::Second, "Second");
                    ui.selectable_value(&mut self.e, Enum::Third, "Third");
                });
        });
    }
}
