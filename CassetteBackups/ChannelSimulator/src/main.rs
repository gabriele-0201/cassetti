mod cassette_simulator;
mod channel_simulation;
mod gui;

use eframe::egui;
use hound;
//use Dsp::signal::signal_vec::SignalPieceVec;
use Dsp::SignalDynamic::Signal;
use FileSystemManager::FileSystemManager;
use ModulationsManagerDynamic::{traits::Modulator, *};

const AUDIO_FILE_NAME: &str = "file.wav";
const AUDIO_FILE_NAME_NOISE: &str = "file_with_noise.wav";

fn main() {
    println!("Hello! This is a Channel Simulator!");

    println!("For now NO input is requested to yuo because I'm lazy");

    // Read the File
    let bytes = FileSystemManager::read("in_test_2_bytes.org").expect("Impossible read");

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Channel Simulator",
        options,
        Box::new(|_cc| Box::new(gui::Gui::new(bytes))),
    )
    .expect("Something went wrong in the window")

    //cassette_simulator::pipewire_test();
}

// This struct will be used to provide EVERY information to the
// GUI to show everything
struct ChannelOutput {
    moduled_signal: Signal,
    moduled_signal_after_channel: Signal,
    demoduled_signal: Signal,
    demoduled_bytes: Vec<u8>,
    samples_per_symbol: usize,
}

impl core::fmt::Debug for ChannelOutput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "CHANNEL OUTPUT")
    }
}

fn channel_simulator(
    bytes: &Vec<u8>,
    modulation: &AvaiableModulation,
    noise_variance: f32,
    delay: f32,
    additional_end_time: f32,
) -> Result<ChannelOutput, &'static str> {
    // Add ErrorCorrection

    // Clone is not the best solution but if I want
    // to use try_from to get the real modulation
    // than it does not make sense to have a reference
    let modulation =
        Modulation::try_from(modulation.clone()).map_err(|_| "IMP create modulation")?;

    // Module
    let mut moduled_signal = modulation.module(&bytes.clone()).map_err(|err| {
        println!("err: {err:?}");
        "IMP modulation"
    })?;

    // SIMULATE CHANNEL

    // -> AWGN
    let mut moduled_signal_after_channel = moduled_signal.clone();
    channel_simulation::add_delay(&mut moduled_signal_after_channel, delay);
    channel_simulation::add_additional_samples(
        &mut moduled_signal_after_channel,
        additional_end_time,
    );
    channel_simulation::add_noise_awgn(
        &mut moduled_signal_after_channel,
        // multily by the rate to obtain the corret variance to apply
        // to every symbol
        noise_variance as f64 * modulation.rate() as f64,
    );

    // FOR NOW AVOID SAVE INTO WAV FILE
    /*

    // NEED to adapt the signal between 0 and 1
    // (due to .wav file that accept only values between zero and one)
    // expect that this is also the minimum
    let max = *moduled_signal
        .inner_ref()
        .iter()
        .max_by(|x, y| {
            x.abs()
                .partial_cmp(&y.abs())
                .expect("Impossible comparison with NaN")
        })
        .ok_or("IMP find max")?;
    //.map_err(|_| "IMP find max")?;

    moduled_signal.apply_function(|v| *v = *v / max);
    moduled_signal_after_channel.apply_function(|v| *v = *v / max);

    // Set up wav channel specification
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: modulation.rate() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let amplitude = i16::MAX as f32;
    // TODO: THIS IS REALLY STUPID, HOW THIS CAN WORK??

    // Save to .wav
    let write_to_file = |name: &str, signal: &Signal| -> Result<(), &'static str> {
        let mut writer = hound::WavWriter::create(name, spec)
            // REALLY THERE'S NO WAY TO DO SOMETHING LIKE THIS?!?!?
            //.map_err(|_| format!("ERROR create wav file {name}"))?;
            .map_err(|_| "ERROR creating wav file")?;
        // TODO: find a way to avoid this clone,
        // (this should be something with .iter and not .into_iter)
        for sample in signal.inner_ref() {
            writer
                .write_sample((sample * amplitude) as i16)
                .expect("Impossible write sample")
        }
        writer
            .finalize()
            //.map_err(|_| format!("ERROR save wav file {name}").as_str())?;
            .map_err(|_| "ERROR saving wav file")?;
        Ok(())
    };

    write_to_file(AUDIO_FILE_NAME, &moduled_signal)?;
    write_to_file(AUDIO_FILE_NAME_NOISE, &moduled_signal_after_channel)?;

    // Read from .wav
    let mut reader = hound::WavReader::open(AUDIO_FILE_NAME_NOISE)
        .map_err(|_| "IMP find file wav to demodule")?;
    let mut moduled_bytes_from_wav: Signal = (
        reader
            .samples::<i16>()
            .map(|s| {
                s.expect("ERROR Impossible get sample from wav file") as f32 / amplitude as f32
            })
            .collect::<Vec<f32>>(),
        modulation.rate(),
    )
        .into();

    // BEFORE demoduling the signal must be adjusted
    moduled_bytes_from_wav.apply_function(|v| *v = *v * max);
    */

    // Demodule
    // TODO: is really ok to demodule from the .wav file?
    let demoduled_bytes = modulation
        .demodule(
            /*moduled_bytes_from_wav*/ moduled_signal_after_channel.clone(),
        )
        // TODO print this error
        .map_err(|_err| "IMP demodule {:?}")?;

    // Resolve error correction

    // Write back the file -> this will manage by the gui
    //FileSystemManager::write(demoduled_bytes).expect("Impossible write");

    Ok(ChannelOutput {
        moduled_signal,
        moduled_signal_after_channel,
        // TODO: this is not the better solution but should work for now
        demoduled_signal: modulation
            .module(&demoduled_bytes)
            .map_err(|_| "Impossibel REmodule")?,
        demoduled_bytes,
        samples_per_symbol: modulation.samples_per_symbol(),
    })
}

struct SNROutput {
    // Here I have to use f64 because the egui accept only this type
    points: Vec<[f64; 2]>,
}

// This function will:
// 1.define a random number of bytes as input
// (always multiple of the numer of bits per symbol of the modualtion, no padding)
//
// 2. iterate over various values of variance:
//  modulation
//  add awgn noise
//  demodulation
//
// 3. plot the result with logarithmic axis
// BER - SNR (Eb/N0 = EnergyPerBit / 2NoiseVariance)
fn calc_snr(
    modulation: &AvaiableModulation,
    n_bytes_to_send: usize,
    snrdb_lower: f32,
    snrdb_upper: f32,
    snrdb_step: f32,
    rep_per_step: usize,
) -> Result<SNROutput, &'static str> {
    let mut snr_points = vec![];

    // Define the Modulation that will be used in the SNR
    let modulation =
        Modulation::try_from(modulation.clone()).map_err(|_| "IMP create modulation")?;

    let average_bit_energy =
        dbg!(modulation.get_average_symbols_energy() as f64) / modulation.bit_per_symbol() as f64;

    // Spawn N random BYTES
    // TODO: check that there are no problem with the modulation of the stuff -> should be implemented PADDING
    let bytes = get_bytes(n_bytes_to_send);
    let total_bit_emitted = (n_bytes_to_send * 8) as f64;

    // Module
    let moduled_signal = modulation.module(&bytes).map_err(|_| "IMP modulation")?;

    let mut snrdb = snrdb_lower;
    while snrdb <= snrdb_upper {
        let mut tot_bit_err = 0;
        let mut n_rep = 0;
        for i in 0..rep_per_step {
            n_rep = dbg!(i);
            let mut to_demodule_signal = moduled_signal.clone();

            // snrdb = 10*log_10(Es / N0) = 10*log_10(Es / (2 * variance))
            let variance: f64 = average_bit_energy / (2. * 10_f64.powf(snrdb as f64 / 10.));

            /* TEST
            let snrdb_test = 10. * (average_bit_energy / (2. * dbg!(variance))).log10();
            //let snrdb_test = 10. * (average_bit_energy / (2. * variance)).log10();
            assert_eq!(snrdb as f64, snrdb_test.round());
            END TEST */

            // multiply the variance by the rate to optain the correct variance
            // to apply to every symbol of the signal
            let sig_variance = variance * modulation.rate() as f64;

            channel_simulation::add_noise_awgn(&mut to_demodule_signal, sig_variance);

            let demoduled_bytes = modulation.demodule(to_demodule_signal).map_err(|_| {
                println!("variance level IMP demodule: {}", variance);
                "IMP demodule, probably due to sync not found"
            })?;

            let errors: f64 = bytes
                .iter()
                .zip(demoduled_bytes.iter())
                .map(|(m, dm)| {
                    let (diff, mut errs) = (m ^ dm, 0);
                    (0..7).for_each(|i| errs += ((diff & (1 << i)) >> i) as usize);
                    errs
                })
                .sum::<usize>() as f64;

            tot_bit_err += dbg!(errors) as usize;
            if dbg!(tot_bit_err) >= 100 {
                break;
            }
        }

        let error_percentage = tot_bit_err as f64 / (total_bit_emitted * (n_rep + 1) as f64);

        snr_points.push([dbg!(snrdb as f64), dbg!(dbg!(error_percentage).log10())]);

        snrdb += snrdb_step;
    }

    Ok(SNROutput { points: snr_points })
}

fn get_bytes(n_bytes: usize) -> Vec<u8> {
    use rand_distr::{Distribution, Uniform};
    let uniform = Uniform::<u8>::new(0, 255);

    let mut res = vec![];

    for _ in 0..n_bytes {
        res.push(uniform.sample(&mut rand::thread_rng()));
    }

    res
}

/*
fn y_log_scale(val: f64, core::ops::RangeInclusive<f64>) -> String {

}
*/
