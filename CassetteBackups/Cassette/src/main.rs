mod audio;

use audio::Audio;
use clap::{Parser, ValueEnum};
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
}

fn main() {
    let cli = Cli::parse();

    print!("Prepare modulation...");
    let sample_rate = 44100;
    let bpsk = BPSK::new(
        1500.0,      // freq
        0.1,         // symbol_period
        sample_rate, // rate
        vec![
            false, true, false, true, true, false, true, true, false, false,
        ], // sync vec
        0.495,       // sync symbol accepted distance
    );
    println!("done!");

    match cli.action {
        Action::Play => {
            play(bpsk);
        }
        Action::Rec => {
            record(bpsk);
        }
        Action::DemodFromWav => {
            demodule_from_wav(bpsk);
        }
    }
}

fn play<T: ModDemod>(modulation: T) {
    print!("Setting up Audio... ");
    let audio = Audio::try_new(modulation.rate()).expect("Impossible create Audio abstraction");
    println!("done!");

    print!("Reading file.. ");
    let bytes = FileSystemManager::read().expect("Impossible read");
    println!("done!");

    print!("Modulating...");
    let file_signal = modulation.module(&bytes).expect("Impossible module");
    println!("done!");

    let stdin = std::io::stdin();

    println!("Press ENTER when you are ready to play");
    stdin
        .read_line(&mut String::new())
        .expect("IMP work with stdin");

    print!("Play modulated file...");
    audio.play(file_signal).expect("Impossible play");
    println!("done!");
}

fn record<T: ModDemod>(modulation: T) {
    print!("Setting up Audio... ");
    let audio = Audio::try_new(modulation.rate()).expect("Impossible create Audio abstraction");
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

    let recorded_file_signal = recorded_signal_rx.recv().expect("IMP get recorded signal");

    demodule_from_signal(modulation, recorded_file_signal);
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
