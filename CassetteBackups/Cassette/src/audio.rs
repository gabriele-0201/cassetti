use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, FromSample, Sample, SampleFormat};
use rand_distr::{Distribution, Normal};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use Dsp::SignalDynamic::Signal;

/// Abstraction layer on audio connection
///
/// You will create an Audio abstraction and than you will be able to:
///
/// record a specified duration into a wav file
///
/// play a wav file
///
/// TODO: record and play a Signal

pub const RECORD_FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
const PLAY_FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/to_play.wav");
type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

pub struct Audio {
    rate: usize,
    host: cpal::Host,
    device: cpal::Device,
    output_supported_config: cpal::SupportedStreamConfig,
    output_config: cpal::StreamConfig,
    input_supported_config: cpal::SupportedStreamConfig,
    input_config: cpal::StreamConfig,
    wav_spec: hound::WavSpec,
}

impl Audio {
    // TODO: no ubstraction on the channels number, for now only ONE
    // maybe later I will ad the possibility of making two
    pub fn try_new(/*channels: usize,*/ rate: usize) -> Result<Self, &'static str> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("IMP get default device")?;

        println!(
            "Input device: {}",
            device.name().expect("IMP get device name")
        );

        // Try to create our own custom SupportedStreamConfig
        // There should be a way to check if they are valid or not,
        // for now let's just create those and try

        let input_supported_config = cpal::SupportedStreamConfig::new(
            //channels as _,
            1,
            cpal::SampleRate(rate as _),
            //TEST
            cpal::SupportedBufferSize::Unknown,
            cpal::SampleFormat::F32,
        );
        let output_supported_config = input_supported_config.clone();

        dbg!(device.default_output_config().unwrap());

        let wav_spec = hound::WavSpec {
            //channels: channels as _,
            channels: 1,
            sample_rate: rate as _,
            //bits_per_sample: (config.sample_format().sample_size() * 8) as _,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        Ok(Self {
            rate,
            host,
            device,
            input_config: input_supported_config.clone().into(),
            input_supported_config,
            output_config: output_supported_config.clone().into(),
            output_supported_config,
            wav_spec,
        })
    }

    /// Record for the diven duration and save the output into a wav file
    /// return a Signal representing the recorded audio
    pub fn record(
        &self,
        stop_handler: std::sync::mpsc::Receiver<()>,
    ) -> Result<Signal, &'static str> {
        // SupportedStreamConfig => StreamConfig
        // and updata channels number to 1

        // The WAV file we're recording to.
        //let spec = Self::wav_spec_from_config(&self.input_supported_config);
        let writer =
            hound::WavWriter::create(RECORD_FILE_PATH, self.wav_spec).expect("IMP create wav file");
        let writer = Arc::new(Mutex::new(Some(writer)));

        // A flag to indicate that recording is in progress.
        println!("Begin recording...");

        // Run the input stream on a separate thread.
        let writer_2 = writer.clone();

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let signal_vec = Arc::new(Mutex::new(Some(Vec::<f32>::new())));
        let mut signal_vec_2 = signal_vec.clone();

        // The only SampleFormat supported is F32, if the device does not support this type
        // of stream than I will return Error
        let stream = self
            .device
            .build_input_stream(
                &self.input_config,
                //move |data, _: &_| Self::write_input_data::<f32, f32>(data, &writer_2),
                move |data: &[f32], _: &_| {
                    if let (Ok(mut guard_w), Ok(mut guard_s)) =
                        (writer_2.try_lock(), signal_vec_2.try_lock())
                    {
                        if let (Some(writer_wav), Some(sig)) = (guard_w.as_mut(), guard_s.as_mut())
                        {
                            for &sample in data.iter() {
                                let sample: f32 = f32::from_sample(sample);
                                // write to the file
                                writer_wav.write_sample(sample).ok();
                                sig.push(sample);
                            }
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|_| "IMP build input stream")?;

        stream.play().map_err(|_| "IMP play stream")?;

        //std::thread::sleep(duration);
        stop_handler.recv().unwrap();
        drop(stream);

        writer
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .finalize()
            .expect("IMP finalize wav file");

        println!("Recording {} complete!", RECORD_FILE_PATH);
        let vec = signal_vec.lock().unwrap().take().unwrap();
        Ok(Signal::from_vec(vec, self.rate))
    }

    /*
    fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
        if format.is_float() {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        }
    }

    fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
        hound::WavSpec {
            channels: config.channels() as _,
            sample_rate: config.sample_rate().0 as _,
            bits_per_sample: (config.sample_format().sample_size() * 8) as _,
            sample_format: Self::sample_format(config.sample_format()),
        }
    }
    */

    /*
    fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
    where
        T: Sample,
        U: Sample + hound::Sample + FromSample<T>,
    {
        if let Ok(mut guard) = writer.try_lock() {
            if let Some(writer) = guard.as_mut() {
                for &sample in input.iter() {
                    let sample: U = U::from_sample(sample);
                    writer.write_sample(sample).ok();
                }
            }
        }
    }
    */

    // Play the Signal and save the relative wav file under PLAY_FILE_PATH
    // could return err if the signal specified in the signal si different
    // from the one in Audio struct
    pub fn play(&self, sig: Signal) -> Result<(), &'static str> {
        if sig.rate() != self.rate {
            return Err("Impossible play signal with rate different from the audio stream");
        }

        // SETUP AUDIO
        let (complete_tx, complete_rx) = std::sync::mpsc::sync_channel(1);

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

        let mut sig_iter = sig.inner().into_iter();

        println!("Begin play...");
        let stream = self
            .device
            .build_output_stream(
                &self.output_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // I use mut just because I think that in this way the data is moved and NOT cloned in the process
                    //let mut signal_iter = sig.inner_ref_mut().iter_mut();
                    for sample in data.iter_mut() {
                        match sig_iter.next() {
                            Some(val) => *sample = val,
                            None => {
                                complete_tx.try_send(()).ok();
                                *sample = Sample::EQUILIBRIUM;
                            }
                        }
                    }
                },
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();
        complete_rx.recv().unwrap();
        println!("Play done!");
        drop(stream);

        Ok(())
    }

    /*
    fn noise_test() -> Result<f32, &'static str> {
        /*
            let time: f32 = 1.0; //sec

            let spec_channels = 1;
            let spec_sample_rate: usize = 44100; // 44kHz

            let noise_variance: f32 = 1.0;

            let zero_signal = Signal::new(
            &|_| 0.0,
            spec_sample_rate,
            (time * spec_sample_rate as f32) as usize,
        );

            let normal = Normal::new(0.0, noise_variance.sqrt()).unwrap();
            let mut noise_signal = Signal::new(
            &|_| normal.sample(&mut rand::thread_rng()),
            spec_sample_rate,
            (time * spec_sample_rate as f32) as usize,
        );
             */
    }
    */

    /* Write to file

        let recorded_signal: Signal = ...;
        let wav_spec = hound::WavSpec {
        //channels: channels as _,
        channels: 1,
        sample_rate: 44100 as _,
        //bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer =
        hound::WavWriter::create("test_recording.wav", wav_spec).expect("IMP create wav file");
    for sample in recorded_signal.inner_ref() {
        writer.write_sample(*sample).ok();
    }
    writer.finalize().expect("Impossible finalize");

    */
}
