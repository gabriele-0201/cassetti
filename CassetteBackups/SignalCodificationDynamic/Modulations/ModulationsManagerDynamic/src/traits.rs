use super::*;

pub trait Modulator {
    fn rate(&self) -> usize;
    fn samples_per_symbol(&self) -> usize;
    fn module(&self, input: &Vec<u8>) -> Result<Signal, ModErr>;
    fn demodule(&self, input: Signal) -> Result<Vec<u8>, DemodErr>;
}
