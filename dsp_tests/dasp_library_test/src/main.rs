use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp_signal::{self as signal, Signal};
use dasp_slice::ToFrameSliceMut;

fn main() {
    // Find and load the wav.
    let reader = hound::WavReader::open(std::path::Path::new("resta_qui.wav")).unwrap();
    let spec = reader.spec();

    // Read the interleaved samples and convert them to a signal.
    let samples = reader.into_samples::<i16>().filter_map(Result::ok);
    let mut frames = signal::from_interleaved_samples_iter(samples).until_exhausted();

    // Initialise CPAL.
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");

    // Create a stream config to match the wave format.
    //
    // NOTE: It's possible that the platform will not support the sample format, sample rate or
    // channel layout of the WAV file. In these cases, you may need to convert the data read from
    // the WAV file to a format compatible with one of the platform's supported stream
    // configurations.
    let config = cpal::StreamConfig {
        channels: spec.channels,
        sample_rate: cpal::SampleRate(spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // A channel for indicating when playback has completed.
    let (complete_tx, complete_rx) = std::sync::mpsc::sync_channel(1);

    // Create and run the CPAL stream.
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let data_fn = move |data: &mut [i16], _info: &cpal::OutputCallbackInfo| {
        let buffer: &mut [[i16; 2]] = data.to_frame_slice_mut().unwrap();
        for out_frame in buffer {
            match frames.next() {
                Some(frame) => *out_frame = frame,
                None => {
                    complete_tx.try_send(()).ok();
                    *out_frame = dasp::Frame::EQUILIBRIUM;
                }
            }
        }
    };
    let stream = device
        .build_output_stream(
            &config,
            data_fn,
            err_fn,
            Some(std::time::Duration::from_secs(60 * 3)),
        )
        .expect("Impossibel build output stream");
    stream.play().unwrap();

    // Block until playback completes.
    complete_rx.recv().unwrap();
    stream.pause().ok();
}
