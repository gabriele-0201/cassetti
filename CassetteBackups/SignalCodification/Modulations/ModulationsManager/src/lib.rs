use Dsp::signal::signal_vec::SignalPieceVec;
use SignalCodification::{traits::*, *};
use BPSK::BPSK;

pub mod traits;
use traits::Modulator;

// Manage HERE all the stuff related to different modulation,
// on the higher level should not change ANYTHING... maybe
#[derive(PartialEq, Debug, Clone)]
pub enum AvaiableModulation {
    BPSK { freq: f32 },
    MQAM { m: usize },
}

// HOW can I create a general manager?

// I'm used a boxed field BECAUSE dyn will require a trait bound in runtime
// but this means that there's no possible way to detect the size of the field (no Sized trait),
// this means that this can't be allocated in the Stack and this means that you can't
// do most of the thigs you do with a standard struct
// The box will give a fixed size to the struct in stack BUT a dynamic size
// of the field in the heap
pub struct Modulation<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize>(
    Box<dyn ModDemod<SAMPLES, RATE, NSYMBOLS>>,
);

impl<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize> TryFrom<AvaiableModulation>
    for Modulation<SAMPLES, RATE, NSYMBOLS>
{
    type Error = &'static str;

    fn try_from(selected_mod: AvaiableModulation) -> Result<Self, Self::Error> {
        Ok(Self(match selected_mod {
            AvaiableModulation::BPSK { freq } => {
                if NSYMBOLS != 2 {
                    return Err("Number of symbols for a bpsk MUST be 2");
                }
                // I think here is better to use directly 2 otherwise a
                // non correct value of NSYMBOLS would generate the
                // BPSK struct with the constant but than that struct
                // will never be used because the previous if will
                // return Error before
                Box::new(BPSK::<SAMPLES, RATE, NSYMBOLS>::new(freq))
            }
            AvaiableModulation::MQAM { m } => todo!(),
        }))
    }
}

impl<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize> Modulator<RATE>
    for Modulation<SAMPLES, RATE, NSYMBOLS>
{
    fn module(&self, input: &Vec<u8>) -> Result<SignalPieceVec<RATE>, ModErr> {
        self.0.module(input)
    }
    fn demodule(&self, input: SignalPieceVec<RATE>) -> Result<Vec<u8>, DemodErr> {
        self.0.demodule(input)
    }
}
