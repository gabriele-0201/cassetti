// TODO: make sanse to keep the function as a generic parameter?
//struct SignalGenerator<Function: Fn(f32) -> f32>(core::marker::PhantomData<Function>);

pub type Symbol<const SAMPLES: usize, const RATE: usize> = SignalPieceSlice<SAMPLES, RATE>;

// Multiple Frequency Shif Keying
struct MFSK<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize> {
    //pub freq_carry: f32,
    pub bit_per_symbol: usize,
    time: [f32; SAMPLES],
    symbols: [Symbol<SAMPLES, RATE>; NSYMBOLS],
    symbol_period: f32,
}

impl<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize> MFSK<SAMPLES, RATE, NSYMBOLS> {
    // fun is the function used to create the symbols, the first argument is the time
    // and the second is the frequency
    fn new(
        freq_carry: f32,
        delta_freq: f32,
        fun: impl Fn(f32, f32) -> f32,
    ) -> Result<Self, &'static str> {
        if NSYMBOLS < 2 {
            return Err("You need at least two symbols");
        }

        // NSYMBOLS / 2 should be a safe operation because if:
        // + is even -> no prob, half on the left of the carry and half on the right
        // + is odd -> (n_symbols - 1)/2 on the left and on the right, one on the carry

        // the first freq from the lowest is:
        // + if n_symbols is even => carry - delta*(floor(n_symbols / 2) + delta/2)
        // + if n_symbols is odd => carry - delta*(floor(n_symbols / 2))
        //
        // Examples:
        //
        // Carry = 4.0
        // Delta = 1.0
        //
        // NSYMBOLS = 4
        //2.5Hz 3.5Hz   |  4.5Hz 5.5Hz
        // |-----|------|---|-----|
        //            carry
        //
        // NSYMBOLS = 3
        // 3Hz  4Hz  5Hz
        // |-----|----|
        //     carry

        let mut freq: f32 = freq_carry - (delta_freq * (NSYMBOLS / 2) as f32)
            + (((NSYMBOLS + 1) & 1) as f32 * delta_freq / 2.0);

        // I don't really know if here happes one or two clone
        let symbols: [SignalPieceSlice<SAMPLES, RATE>; NSYMBOLS] = core::array::from_fn(|_| {
            freq += delta_freq;

            let fun_with_freq = |time: f32| -> f32 { fun(time, freq) };

            SignalPieceSlice::<SAMPLES, RATE>::new(&fun_with_freq)
        });

        // .floor shold not give any problem
        let bit_per_symbol = (NSYMBOLS as f32).log2().floor() as usize;

        Ok(Self {
            //freq_carry,
            bit_per_symbol,
            time: symbols[0].get_time(),
            symbols,
            symbol_period: (SAMPLES as f32 * (1.0 / RATE as f32)), // NEED to find a more intelligent way to do this
        })
    }

    // REALLY STUPID but for now I use a Vec<u8> to represent a Vec of bits, every value grater than zero is equal to ONE
    // TODO: use crate bit-vec and implement a from vec of bytes to u32 or something like this
    fn module(&self, input: &Vec<u8>) -> Result<Vec<(f32, f32)>, &'static str> {
        // We have N symbols and the approach is ONLY for now:
        // + we take the input and split it in groups on log2(NSYMBOLS)
        // + convert every group in an integer and use that as index in the symbol's array

        let cast_to_integer = |i: &[u8]| -> usize {
            let mut res: usize = 0;
            i.iter().enumerate().for_each(|(index, val)| match val {
                0 => (),
                _ => res += 2usize.pow(index as u32),
            });
            res
        };

        Ok(input
            .chunks(self.bit_per_symbol)
            .enumerate()
            .map(|(index, val)| {
                self.time
                    .clone()
                    .iter()
                    .zip(self.symbols[dbg!(cast_to_integer(val))].clone().into_iter())
                    .map(|(x, y)| (x + index as f32 * self.symbol_period, y))
                    .collect()
            })
            .collect::<Vec<Vec<(f32, f32)>>>()
            .concat())
    }

    fn demodule(&self, input: Vec<f32>) -> (Vec<u8>, Vec<f32>) {
        todo!()
    }
}
