use super::*;

use Dsp::SignalDynamic::Signal;

pub type Symbol = Vec<f32>;

pub trait ModDemod /*<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize>*/ {
    // This method is better to override using an intern value
    // evaluated at instantiation time
    // The number of bit per simbol will be never bigger than 256
    fn bit_per_symbol(&self) -> u8;

    fn rate(&self) -> usize;

    fn samples_per_symbol(&self) -> usize;

    // TODO Idk if this will work
    //fn symbols(&self) -> &[Symbol<SAMPLES, RATE>/*; NSYMBOLS*/];
    fn symbols(&self) -> &[Symbol];

    // Every modulation will have some symbols to properly sync
    // this signal will be created at the creation of the modulation
    // and than used in modulation and demodulation
    fn get_sync(&self) -> Vec<f32>;

    // this method will take a signal, find a sync at the beginning
    // remove so that the demodulation now can start from the beginning
    // of the modulated signal
    fn sync(&self, input: &mut Signal) -> Result<(), DemodErr>;

    // TODO: work better with bit-vec and iterators,
    // could be used the collect of reference approach
    //
    // The return is a SignalPieceVec with the same rate of th
    fn module(&self, input: &Vec<u8>) -> Result<Signal, ModErr> {
        // ADD at the beginnig a u32 to specify the amount of bytes that
        // will be modulated, during the demodulation this value will be used
        // to demodule a specific amount of the remening signal

        // CONVETION: I will push the byte in a little endian order
        // so the first byte to appear in the signal is the least significant
        let num_bytes: u32 = dbg!(input.len() as u32);
        let input = [num_bytes.to_le_bytes().to_vec(), input.clone()]
            .concat()
            .to_vec();

        // We have N symbols and the approach is ONLY for now:
        // + we take the input and split it in groups on log2(NSYMBOLS)
        // + convert every group in an integer and use that as index in the symbol's array

        let raw_symbols = RawSymbols::try_get_symbols(&input, self.bit_per_symbol())
            .map_err(|_| ModErr::InvalidInput)?;

        // The access to the array should never panic because
        // raw_symbols is already entirely checked based on bit_per_symbol
        let modulated_signal = raw_symbols
            .into_iter()
            .map(|n_symbol| self.symbols()[n_symbol].clone())
            .collect::<Vec<Vec<f32>>>()
            .concat();

        // A new function is required, sync
        // this will return the symbols that are needed to add
        // at the beginnig of the modulated symbol
        Ok(Signal::from_vec(
            [self.get_sync(), modulated_signal].concat(),
            self.rate(),
        ))
    }

    // What this method does is:
    // 1. search the sync signal
    // 2. demodule the first 4bytes (=4*8*symbol_period*sample_rate samples) to the
    // the number of exepcted bytes to demodule
    // (return Err if the expected bytes is less than the avaiable signal)
    // 3. demodule the just defined amount of signal
    // 4. return the bytes
    fn symbols_demodulation(&self, input: Signal) -> Result<Vec<usize>, DemodErr>;

    // This is a little bit more complicated I don't know if this can be generalized
    // TODO: this can be generalized, the modulation shoul thouch ONLY symbols
    // and the bytes are managed by the trait
    fn demodule(&self, input: Signal) -> Result<Vec<u8>, DemodErr> {
        let raw_bytes = self.symbols_demodulation(input)?;

        collect_bytes_from_raw_bytes(raw_bytes, self.bit_per_symbol())
            .map_err(|_| DemodErr::InvalidInput)
    }
}
