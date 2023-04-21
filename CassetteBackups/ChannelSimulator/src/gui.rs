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
    n_symbols_in_modulation: usize,
    n_symbols_in_sync_signal: usize,
    snr_output: Result<SNROutput, &'static str>,
    snr_n_bytes: usize,
    snrdb_upper: f32,
    snrdb_lower: f32,
    snrdb_step: f32,
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
                symbol_period: 1.,
                rate: 100,
                freq: 1.,
                sync_symbols: vec![],
                acceptance_sync_distance: 0.01,
                use_expected_bytes: false,
            },
            channel_output: Err("Modulation TBD"),
            n_symbols_to_show: 5,
            string_sync_symbols: String::from(""),
            n_symbols_in_modulation: 2,
            n_symbols_in_sync_signal: 10,
            snr_output: Err("SNR TBD"),
            snr_n_bytes: 10000,
            snrdb_upper: 0.,
            snrdb_lower: 0.,
            snrdb_step: 1.,
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
                            symbol_period: 1.,
                            rate: 100,
                            freq: 1.,
                            sync_symbols: vec![],
                            acceptance_sync_distance: 0.01,
                            use_expected_bytes: false,
                        },
                        "BPSK",
                    );
                    ui.selectable_value(
                        &mut self.modulation,
                        AvaiableModulation::MQAM {
                            symbol_period: 1.,
                            rate: 100,
                            freq: 1.,
                            m: 16,
                            sync_symbols: vec![],
                            acceptance_sync_distance: 0.01,
                            use_expected_bytes: false,
                        },
                        "MQAM",
                    );
                });

            macro_rules! s_period_and_rate_slider {
                ($s: ident, $r: ident) => {
                    ui.add(egui::Slider::new($s, 0.0..=1.0).text("Symbol Period"));
                    // 0..100kHz
                    ui.add(egui::Slider::new($r, 0..=100000).text("Sampling Freq. - Rate"));
                };
            }

            macro_rules! sync_and_expected_bytes {
                ($sync: ident, $acc_distance: ident, $exp: ident) => {
                    ui.label("Sync symbols");
                    let sync_symbols_changing =
                        ui.add(egui::TextEdit::singleline(&mut self.string_sync_symbols));
                    if sync_symbols_changing.changed() {
                        //.expect("unexpected sync symbol")
                        let parsing_sync_symbols: Result<Vec<usize>, _> = self
                            .string_sync_symbols
                            .as_str()
                            .split(",")
                            .map(|s| s.parse::<usize>())
                            .collect();
                        if let Ok(res) = parsing_sync_symbols {
                            *$sync = res;
                        }
                    }

                    ui.label("Number of symbols, used by random sync signal");
                    ui.add(egui::DragValue::new(&mut self.n_symbols_in_modulation));
                    ui.label("Number of symbols in the sync signal");
                    ui.add(egui::DragValue::new(&mut self.n_symbols_in_sync_signal));
                    if ui.button("use random sync signal").clicked() {
                        use rand_distr::{Distribution, Uniform};
                        let unifomr = Uniform::new(0, self.n_symbols_in_modulation);

                        *$sync = vec![];
                        for _ in 0..self.n_symbols_in_sync_signal {
                            $sync.push(unifomr.sample(&mut rand::thread_rng()))
                        }

                        self.string_sync_symbols = $sync
                            .iter()
                            .map(|v| format!("{},", v.to_string()))
                            .collect();
                        self.string_sync_symbols.pop();
                    }

                    ui.add(
                        egui::Slider::new($acc_distance, 0.0..=0.3)
                            .text("Sync signal distance acceptance"),
                    );
                    ui.add(egui::Checkbox::new($exp, "Number expected bytes"));
                };
            }

            match &mut self.modulation {
                AvaiableModulation::BPSK {
                    ref mut symbol_period,
                    ref mut rate,
                    ref mut freq,
                    ref mut sync_symbols,
                    ref mut acceptance_sync_distance,
                    ref mut use_expected_bytes,
                } => {
                    s_period_and_rate_slider!(symbol_period, rate);
                    // Frequency of the BPSK can go from 0 to 20kHz
                    ui.add(egui::Slider::new(freq, 0.0..=20000.0).text("BPSK Frequency"));

                    sync_and_expected_bytes!(
                        sync_symbols,
                        acceptance_sync_distance,
                        use_expected_bytes
                    );
                }
                AvaiableModulation::MQAM {
                    ref mut symbol_period,
                    ref mut rate,
                    ref mut freq,
                    ref mut m,
                    ref mut sync_symbols,
                    ref mut acceptance_sync_distance,
                    ref mut use_expected_bytes,
                } => {
                    s_period_and_rate_slider!(symbol_period, rate);
                    ui.add(egui::Slider::new(freq, 0.0..=20000.0).text("QAM Frequency"));
                    ui.add(
                        egui::Slider::new(m, 4..=256)
                            .text("Number of symbols, must be an even power of two"),
                    );
                    sync_and_expected_bytes!(
                        sync_symbols,
                        acceptance_sync_distance,
                        use_expected_bytes
                    );
                }
            };

            ui.collapsing("Modulation and Demodulation Test", |ui| {
                ui.add(
                    egui::Slider::new(&mut self.noise_variance, 0.0..=200.0).text("Noise Variance"),
                );
                ui.add(egui::Slider::new(&mut self.delay, 0.0..=5.0).text("Delay"));
                ui.add(
                    egui::Slider::new(&mut self.additional_end_time, 0.0..=5.0)
                        .text("Additional end time"),
                );

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
                            .map(|(m, dm)| {
                                let (diff, mut errs) = (m ^ dm, 0);
                                (0..7).for_each(|i| errs += ((diff & (1 << i)) >> i) as usize);
                                errs
                            })
                            .sum();

                        ui.label(format!(
                            "Error percentage: {}",
                            errors as f32 / (self.bytes.len() * 8) as f32
                        ));
                    }
                    Err(ref err) => {
                        ui.label(format! {"channel err: {}", err});
                    }
                }
            });

            ui.collapsing("SNR", |ui| {
                ui.add(egui::Slider::new(&mut self.snr_n_bytes, 1..=10000000).text("Bytes number"));
                ui.add(
                    // TODO: could be usefull to print also the relative Es and Variance near the selected SNR
                    egui::Slider::new(&mut self.snrdb_upper, 0.0..=50.)
                        .text(format!("Upper bound SNR (inclusive)")),
                );
                ui.add(
                    egui::Slider::new(&mut self.snrdb_lower, -50.0..=0.0).text("Lower bound SNR"),
                );
                ui.add(egui::Slider::new(&mut self.snrdb_step, 0.001..=10.).text("SNR step"));

                if ui.button("Calc new SNR").clicked() {
                    self.snr_output = Err("SNR TBD");
                }

                if let Err("SNR TBD") = self.snr_output {
                    println!("START SNR");
                    self.snr_output = calc_snr(
                        &self.modulation,
                        self.snr_n_bytes,
                        self.snrdb_lower,
                        self.snrdb_upper,
                        self.snrdb_step,
                        100,
                    );
                    println!("END SNR");
                }

                match self.snr_output {
                    Ok(ref snr_output) => {
                        ui.heading("SNR");

                        // PLOT moduleted signal
                        Plot::new("Moduled Bytes")
                            //.view_aspect(2.0)
                            .auto_bounds_y()
                            //.height(plot_height)
                            .show(ui, |plot_ui| {
                                plot_ui.line(Line::new(snr_output.points.clone()))
                            });
                    }
                    Err(ref err) => {
                        ui.label(format! {"channel err: {}", err});
                    }
                }
            });
            // Button for save into file demoduled_bytes
        });
    }
}
