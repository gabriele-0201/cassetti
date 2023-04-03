use super::*;
use eframe::egui::{
    self,
    plot::{Line, Plot},
};

pub fn find_max_amplitude(rate: usize) {
    // Create signal with different aplitudes

    let freq = 1000.;
    let sec_per_amplitude: f32 = 1.;
    // amplitued step MUST be a divisor of 1
    let amplitude_step: f32 = 0.01;
    let n_amplituedes = 0.05 / amplitude_step;
    assert_eq!(n_amplituedes.floor(), n_amplituedes);
    let amplitudes = Signal::new(
        &|t| {
            ((t % sec_per_amplitude as f32) * 2. * std::f32::consts::PI * freq).sin()
                * ((t / sec_per_amplitude).floor() + 1.)
                * amplitude_step
        },
        rate,
        (n_amplituedes * sec_per_amplitude * rate as f32) as usize,
    );
    println!("Amplituedes DONE");

    // Play signal
    play(amplitudes.clone());

    // Record Signal
    let recorded_amplitudes = record(rate);
    //let recorded_amplitudes = Signal::from_vec(vec![], rate);

    // Plot Signal
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(Gui::new(amplitudes, recorded_amplitudes))),
    )
    .expect("Something went wrong in the window")

    // TODO: Create something automatically to decide the max value of amplitude
}

pub struct Gui {
    amplitudes: Vec<[f64; 2]>,
    recorded_amplitudes: Vec<[f64; 2]>,
}

impl Gui {
    fn new(amplitudes: Signal, recorded_amplitudes: Signal) -> Self {
        // SUB sampling to make plotter faster
        // to 1000 of rate
        let to_skip = amplitudes.rate() / 10000;
        let get_points = |sig: Signal| {
            sig.get_coordinates(None)
                .into_iter()
                .enumerate()
                .filter_map(|(i, val)| if i % to_skip == 0 { Some(val) } else { None })
                .collect()
        };
        Self {
            amplitudes: get_points(amplitudes),
            recorded_amplitudes: get_points(recorded_amplitudes),
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.heading("10 different recorded amplitude");

            let plot_height = ui.available_height() / 2.3;

            Plot::new("To record Amplitudes")
                .auto_bounds_y()
                .height(plot_height)
                .show(ui, |plot_ui| {
                    plot_ui.line(Line::new(self.amplitudes.clone()))
                });

            Plot::new("Recorded Amplitues")
                .auto_bounds_y()
                .height(plot_height)
                .show(ui, |plot_ui| {
                    plot_ui.line(Line::new(self.recorded_amplitudes.clone()))
                });
        });
    }
}
