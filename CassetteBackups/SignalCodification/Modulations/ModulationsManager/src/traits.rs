use super::*;

pub trait Modulator<const RATE: usize> {
    fn module(&self, input: &Vec<u8>) -> Result<SignalPieceVec<RATE>, ModErr>;
    fn demodule(&self, input: SignalPieceVec<RATE>) -> Result<Vec<u8>, DemodErr>;
}
