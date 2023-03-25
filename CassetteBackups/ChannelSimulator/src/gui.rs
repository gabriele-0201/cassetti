use super::*;

use egui::plot::{Line, Plot, PlotPoints};

pub struct Gui {
    bytes: Vec<u8>,
    noise_variance: f32,
    delay: f32,               // seconds
    additional_end_time: f32, // seconds
    modulation: AvaiableModulation,
    channel_output: Result<ChannelOutput, &'static str>,
    n_symbols_to_show: usize,
    string_sync_symbols: String,
}

impl Gui {
    // Bytes will be removed from here when I will add
    // a widget to select what bytes I want to send
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            noise_variance: 0.0,
            delay: 0.0,
            additional_end_time: 0.0,
            // Default Modulation
            modulation: AvaiableModulation::BPSK {
                symbol_period: 0.01,
                rate: 44100,
                freq: 100.0,
                sync_symbols: vec![0, 1],
                acceptance_sync_distance: 0.01,
            },
            channel_output: Err("Modulation TBD"),
            n_symbols_to_show: 5,
            string_sync_symbols: String::from("0,1"),
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.heading("Modulation Managment");

            // Select modulation
            egui::ComboBox::from_label("")
                .selected_text(format!("{}", self.modulation))
                .show_ui(ui, |ui| {
                    // Here are selected the default values of the modulations
                    ui.selectable_value(
                        &mut self.modulation,
                        AvaiableModulation::BPSK {
                            symbol_period: 0.01,
                            rate: 44100,
                            freq: 100.0,
                            sync_symbols: vec![0, 1],
                            acceptance_sync_distance: 0.01,
                        },
                        "BPSK",
                    );
                    ui.selectable_value(
                        &mut self.modulation,
                        AvaiableModulation::MQAM {
                            symbol_period: 0.01,
                            rate: 44100,
                            freq: 1000.0,
                            m: 4,
                        },
                        "MQAM",
                    );
                });

            ui.add(egui::Slider::new(&mut self.noise_variance, 0.0..=2.0).text("Noise Variance"));
            ui.add(egui::Slider::new(&mut self.delay, 0.0..=5.0).text("Delay"));
            ui.add(
                egui::Slider::new(&mut self.additional_end_time, 0.0..=5.0)
                    .text("Additional end time"),
            );

            macro_rules! s_period_and_rate_slider {
                ($s: ident, $r: ident) => {
                    ui.add(egui::Slider::new($s, 0.0..=0.1).text("Symbol Period"));
                    // 0..100kHz
                    ui.add(egui::Slider::new($r, 0..=100000).text("Sampling Freq. - Rate"));
                };
            }

            match &mut self.modulation {
                AvaiableModulation::BPSK {
                    ref mut symbol_period,
                    ref mut rate,
                    ref mut freq,
                    ref mut sync_symbols,
                    ref mut acceptance_sync_distance,
                } => {
                    s_period_and_rate_slider!(symbol_period, rate);
                    // Frequency of the BPSK can go from 0 to 20kHz
                    ui.add(egui::Slider::new(freq, 0.0..=20000.0).text("BPSK Frequency"));
                    ui.label("Sync symbols");
                    let sync_symbols_changing =
                        ui.add(egui::TextEdit::singleline(&mut self.string_sync_symbols));
                    if sync_symbols_changing.changed() {
                        //.expect("unexpected sync symbol")
                        let parsing_sync_symbols: Result<Vec<u8>, _> = self
                            .string_sync_symbols
                            .as_str()
                            .split(",")
                            .map(|s| s.parse::<u8>())
                            .collect();
                        if let Ok(res) = parsing_sync_symbols {
                            *sync_symbols = res;
                        }
                    }
                    ui.add(
                        egui::Slider::new(acceptance_sync_distance, 0.0..=0.3)
                            .text("Sync signal distance acceptance"),
                    );
                }
                AvaiableModulation::MQAM {
                    ref mut symbol_period,
                    ref mut rate,
                    ref mut freq,
                    ref mut m,
                } => {
                    s_period_and_rate_slider!(symbol_period, rate);
                    ui.add(egui::Slider::new(freq, 0.0..=20000.0).text("QAM Frequency"));
                    ui.add(
                        egui::Slider::new(m, 4..=256)
                            .text("Number of symbols, must be an even power of two"),
                    );
                }
            };

            if ui.button("Apply new Modulation").clicked() {
                self.channel_output = Err("Modulation TBD");
            }

            if let Err("Modulation TBD") = self.channel_output {
                self.channel_output = channel_simulator(
                    &self.bytes,
                    &self.modulation,
                    self.noise_variance,
                    self.delay,
                    self.additional_end_time,
                );
            }

            match self.channel_output {
                Ok(ref channel_output) => {
                    ui.heading("Signal Analisys");
                    ui.add(
                        egui::Slider::new(&mut self.n_symbols_to_show, 0..=100)
                            .text("Number of Symbols to show"),
                    );

                    let plot_height = ui.available_height() / 3.5;

                    let samples_to_show =
                        self.n_symbols_to_show * channel_output.samples_per_symbol;

                    // PLOT moduleted signal
                    let moduled_signal_line = Line::new(
                        channel_output
                            .moduled_signal
                            .get_coordinates(Some(samples_to_show)),
                    );
                    ui.label("Moduled Bytes");
                    Plot::new("Moduled Bytes")
                        //.view_aspect(2.0)
                        .auto_bounds_y()
                        .height(plot_height)
                        .show(ui, |plot_ui| plot_ui.line(moduled_signal_line));

                    // PLOT moduled signal with noise
                    let moduled_signal_with_noise_line = Line::new(
                        channel_output
                            .moduled_signal_after_channel
                            .get_coordinates(Some(samples_to_show)),
                    );

                    ui.label("Moduled Bytes With Noise");
                    Plot::new("Moduled Bytes With Noise")
                        //.view_aspect(2.0)
                        .auto_bounds_y()
                        .height(plot_height)
                        .show(ui, |plot_ui| plot_ui.line(moduled_signal_with_noise_line));

                    // PLOT demoduled signal
                    let demoduled_signal_line = Line::new(
                        channel_output
                            .demoduled_signal
                            .get_coordinates(Some(samples_to_show)),
                    );
                    ui.label("Demoduled Bytes");
                    Plot::new("Demoduled Bytes")
                        //.view_aspect(2.0)
                        .auto_bounds_y()
                        .height(plot_height)
                        .show(ui, |plot_ui| plot_ui.line(demoduled_signal_line));

                    let errors: usize = self
                        .bytes
                        .iter()
                        .zip(channel_output.demoduled_bytes.iter())
                        .map(|(m, dm)| if m == dm { 0 } else { 1 })
                        .sum();

                    ui.label(format!(
                        "Error percentage: {}",
                        errors as f32 / self.bytes.len() as f32
                    ));
                }
                Err(ref err) => {
                    ui.label(format! {"channel err: {}", err});
                }
            }

            // Button for save into file demoduled_bytes
        });
    }
}
