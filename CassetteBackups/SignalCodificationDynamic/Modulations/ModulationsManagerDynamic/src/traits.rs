use super::*;

pub trait Modulator {
    fn rate(&self) -> usize;
    fn bit_per_symbol(&self) -> u8;
    fn samples_per_symbol(&self) -> usize;
    fn module(&self, input: &Vec<u8>) -> Result<Signal, ModErr>;
    fn demodule(&self, input: Signal) -> Result<Vec<u8>, DemodErr>;
    fn get_average_symbols_energy(&self) -> f32;
}
