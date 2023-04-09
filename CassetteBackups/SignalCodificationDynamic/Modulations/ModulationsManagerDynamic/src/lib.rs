use BPSKDynamic::BPSK;
use Dsp::SignalDynamic::Signal;
use MQAMDynamic::MQAM;
use SignalCodificationDynamic::{traits::*, *};

pub mod traits;
use traits::Modulator;

// Manage HERE all the stuff related to different modulation,
// on the higher level should not change ANYTHING... maybe
#[derive(PartialEq, Debug, Clone)]
pub enum AvaiableModulation {
    BPSK {
        symbol_period: f32,
        rate: usize,
        freq: f32,
        // This will be translated to bool, 0 => false, _ => true
        sync_symbols: Vec<u8>,
        acceptance_sync_distance: f32,
        use_expected_bytes: bool,
    },
    MQAM {
        symbol_period: f32,
        rate: usize,
        freq: f32,
        m: usize,
    },
}

impl std::fmt::Display for AvaiableModulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let name = match self {
            AvaiableModulation::BPSK { .. } => "BPSK",
            AvaiableModulation::MQAM { .. } => "MQAM",
        };
        write!(f, "{}", name)
    }
}

// HOW can I create a general manager?

// I'm used a boxed field BECAUSE dyn will require a trait bound in runtime
// but this means that there's no possible way to detect the size of the field (no Sized trait),
// this means that this can't be allocated in the Stack and this means that you can't
// do most of the thigs you do with a standard struct
// The box will give a fixed size to the struct in stack BUT a dynamic size
// of the field in the heap
pub struct Modulation(Box<dyn ModDemod>);

impl TryFrom<AvaiableModulation> for Modulation {
    type Error = &'static str;

    fn try_from(selected_mod: AvaiableModulation) -> Result<Self, Self::Error> {
        Ok(Self(match selected_mod {
            AvaiableModulation::BPSK {
                symbol_period,
                rate,
                freq,
                sync_symbols,
                acceptance_sync_distance,
                use_expected_bytes,
            } => Box::new(BPSK::new(
                freq,
                symbol_period,
                rate,
                sync_symbols
                    .iter()
                    .map(|v| if *v == 0 { false } else { true })
                    .collect(),
                acceptance_sync_distance,
                use_expected_bytes,
            )),
            AvaiableModulation::MQAM {
                symbol_period,
                rate,
                freq,
                m,
            } => Box::new(MQAM::new(freq, symbol_period, rate, m)),
        }))
    }
}

impl Modulator for Modulation {
    fn rate(&self) -> usize {
        self.0.rate()
    }
    fn samples_per_symbol(&self) -> usize {
        self.0.samples_per_symbol()
    }
    fn module(&self, input: &Vec<u8>) -> Result<Signal, ModErr> {
        self.0.module(input)
    }
    fn demodule(&self, input: Signal) -> Result<Vec<u8>, DemodErr> {
        self.0.demodule(input)
    }
    fn get_average_symbols_energy(&self) -> f32 {
        self.0.get_average_symbols_energy()
    }
}
