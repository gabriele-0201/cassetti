use super::*;

pub type Symbol<const SAMPLES: usize, const RATE: usize> = SignalPieceSlice<SAMPLES, RATE>;

// SAMPLES per symbol
pub trait ModDemod<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize> {
    // This method is better to override using an intern value
    // evaluated at instantiation time
    // The number of bit per simbol will be never bigger than 256
    fn bit_per_symbol(&self) -> u8 {
        (NSYMBOLS as f32).log2().floor() as u8
    }

    // Also this could be reimplemented creating the time only once!
    fn time(&self) -> SignalPieceSlice<SAMPLES, RATE> {
        SignalPieceSlice::get_time()
    }

    // Also this could be reimplemented creating the time only once!
    // TODO: decide with WHICH precision I want to work
    fn symbol_period(&self) -> f32 {
        SAMPLES as f32 * (1.0 / RATE as f32)
    }

    // TODO Idk if this will work
    //fn symbols(&self) -> &[Symbol<SAMPLES, RATE>/*; NSYMBOLS*/];
    fn symbols(&self) -> &[Symbol<SAMPLES, RATE>];

    // TODO: work better with bit-vec and iterators,
    // could be used the collect of reference approach
    //
    // The return is a SignalPieceVec with the same rate of th
    fn module(&self, input: &Vec<u8>) -> Result<SignalPieceVec<RATE>, ModErr> {
        // We have N symbols and the approach is ONLY for now:
        // + we take the input and split it in groups on log2(NSYMBOLS)
        // + convert every group in an integer and use that as index in the symbol's array

        let raw_symbols = RawSymbols::try_get_symbols(input, self.bit_per_symbol())
            .map_err(|_| ModErr::InvalidInput)?;

        Ok(raw_symbols
            .into_iter()
            .map(|n_symbol| self.symbols()[n_symbol].clone())
            .collect())

        /* This is the return when I was returing at the same time
         * time and values of the signal
        Ok(input
            .chunks(self.bit_per_symbol())
            .enumerate()
            .map(|(index, val)| {
                self.time()
                    .into_iter()
                    .zip(self.symbols()[cast_to_integer(val)].clone().into_iter())
                    .map(|(x, y)| (x + index as f32 * self.symbol_period(), y))
                    .collect()
            })
            .collect::<Vec<Vec<(f32, f32)>>>()
            .concat())
        */
    }

    fn symbols_demodulation(&self, input: SignalPieceVec<RATE>) -> Result<Vec<usize>, DemodErr>;

    // This is a little bit more complicated I don't know if this can be generalized
    // TODO: this can be generalized, the modulation shoul thouch ONLY symbols
    // and the bytes are managed by the trait
    fn demodule(&self, input: SignalPieceVec<RATE>) -> Result<Vec<u8>, DemodErr> {
        let raw_bytes = self.symbols_demodulation(input)?;

        collect_bytes_from_raw_bytes(raw_bytes, self.bit_per_symbol())
            .map_err(|_| DemodErr::InvalidInput)
    }
}
