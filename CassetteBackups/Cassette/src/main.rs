/*
 * This terminal based application will allow you to executed different
 * programs, each one will modify or use a common .toml file that will
 * contain all the information needed to make a proper modulation
 * and demodulation of the bytes.
 *
 * Everything reproduced or recorded is also saved in a .wat file
 * to make possibel re-write something on the cassette tape without
 * the need to re-execute the modulation
 * */

mod audio;
mod linear;

use audio::Audio;
use clap::{Parser, ValueEnum};
use linear::find_max_amplitude;
use BPSKDynamic::BPSK;
use Dsp::SignalDynamic::Signal;
use FileSystemManager::FileSystemManager;
use SignalCodificationDynamic::traits::ModDemod;

#[derive(Parser)]
struct Cli {
    action: Action,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Action {
    Play,
    Rec,
    DemodFromWav,
    // play nothing, record nothing, play noise, record noise -> calc. channel noise
    FindChannleNoise,
    // TODO: something that like record sin of different aplitude and than plot it and let the 'user' decide wich one does not clip
    FindMaxAmplitude,
}

fn main() {
    let cli = Cli::parse();

    println!("Prepare modulation...");
    // TODO: this information should be in the .toml file
    let sample_rate = 44100;
    let max_amplitude = 1.0;
    let bpsk = BPSK::new(
        1000.0,      // freq
        0.3,         // symbol_period
        sample_rate, // rate
        vec![
            false, true, true, false, false, true, false, false, true, true,
        ], // sync vec
        0.8,         // sync symbol accepted distance
        true,
    );
    println!("done!");

    match cli.action {
        Action::Play => {
            mod_and_play(bpsk, max_amplitude);
        }
        Action::Rec => {
            record_and_demod(bpsk, max_amplitude);
        }
        Action::DemodFromWav => {
            demodule_from_wav(bpsk);
        }
        Action::FindChannleNoise => {
            unimplemented!("Not implemented YET")
        }
        Action::FindMaxAmplitude => {
            find_max_amplitude(sample_rate);
        }
    }
}

fn mod_and_play<T: ModDemod>(modulation: T, max_amplitude: f32) {
    print!("Reading file.. ");
    let bytes = FileSystemManager::read().expect("Impossible read");
    println!("done!");

    print!("Modulating...");
    let mut file_signal = modulation.module(&bytes).expect("Impossible module");
    println!("done!");

    // compress signal based on the max supported amplitude
    compress_signal(&modulation, &mut file_signal, max_amplitude);

    play(file_signal)
}

fn play(signal: Signal) {
    print!("Setting up Audio... ");
    let audio = Audio::try_new(signal.rate()).expect("Impossible create Audio abstraction");
    println!("done!");

    let stdin = std::io::stdin();

    println!("Press ENTER when you are ready to play");
    stdin
        .read_line(&mut String::new())
        .expect("IMP work with stdin");

    print!("Play modulated file...");
    audio.play(signal).expect("Impossible play");
    println!("done!");
}

fn compress_signal<T: ModDemod>(modulation: &T, sig: &mut Signal, max_amplitude: f32) {
    // TODO: AHHHH Here max (the max value found in modulation) needs to be saved
    let max = modulation.max_value_in_symbols();
    // max_amplitude : max = x: v

    sig.apply_function(|v| *v = (*v * max_amplitude) / max);
}

fn decompress_signal<T: ModDemod>(modulation: &T, sig: &mut Signal, max_amplitude: f32) {
    let max = modulation.max_value_in_symbols();
    // decompress based on max_amplitude
    // max_amplitude : max = x: v

    sig.apply_function(|x| *x = (*x * max) / max_amplitude);
}

fn record_and_demod<T: ModDemod>(modulation: T, max_amplitude: f32) {
    let mut recorded_file_signal = record(modulation.rate());
    decompress_signal(&modulation, &mut recorded_file_signal, max_amplitude);
    demodule_from_signal(modulation, recorded_file_signal);
}

fn record(rate: usize) -> Signal {
    print!("Setting up Audio... ");
    let audio = Audio::try_new(rate).expect("Impossible create Audio abstraction");
    println!("done!");

    let stdin = std::io::stdin();

    println!("Press ENTER to start recording");
    stdin
        .read_line(&mut String::new())
        .expect("IMP work with stdin");

    // set up channle to stop recording
    let (stop_recording_tx, stop_recording_rx) = std::sync::mpsc::sync_channel::<()>(1);
    let (recorded_signal_tx, recorded_signal_rx) = std::sync::mpsc::sync_channel::<Signal>(1);

    print!("Recording file...");
    std::thread::spawn(move || {
        let recorded_file_signal = audio.record(stop_recording_rx).expect("Impossible record");
        recorded_signal_tx
            .try_send(recorded_file_signal)
            .expect("IMP send recorded signal");
    });

    println!("Press ENTER to stop recording");
    stdin
        .read_line(&mut String::new())
        .expect("IMP work with stdin");

    stop_recording_tx.try_send(()).expect("IMP stop recording");

    recorded_signal_rx.recv().expect("IMP get recorded signal")
}

fn demodule_from_signal<T: ModDemod>(modulation: T, signal: Signal) {
    println!("Demodulating...");
    let result_bytes = modulation.demodule(signal).expect("IMP demodule");
    println!("done!");

    println!("Save demodulated file...");
    FileSystemManager::write(result_bytes).expect("Impossible write");
    println!("done!");
}

fn demodule_from_wav<T: ModDemod>(modulation: T) {
    let mut reader =
        hound::WavReader::open(audio::RECORD_FILE_PATH).expect("IMP find file wav to demodule");

    let recorded_signal: Signal = (
        reader
            .samples::<f32>()
            .map(|s| s.expect("ERROR Impossible get sample from wav file") as f32)
            .collect::<Vec<f32>>(),
        modulation.rate(),
    )
        .into();

    demodule_from_signal(modulation, recorded_signal);
}
