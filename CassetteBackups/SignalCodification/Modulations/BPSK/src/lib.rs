use Dsp::signal::{signal_slice::SignalPieceSlice, signal_vec::SignalPieceVec};
use SignalCodification::{traits::*, *};

// Use this just to make the code more clear
const NSYMBOLS: usize = 2;

// MAKE all of this general to the floating precision
pub struct BPSK<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize> {
    time: SignalPieceSlice<SAMPLES, RATE>,
    //symbols: [Symbol<SAMPLES, RATE>; 2],
    // TODO: for now I will use vec Otherwise here is impossible
    // to use a static array of lenght NSYMBOLS because
    // in this case the number of symbols will be always 2
    symbols: Vec<Symbol<SAMPLES, RATE>>,
    symbol_period: f32,
}

impl<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize> BPSK<SAMPLES, RATE, NSYMBOLS> {
    pub fn new(freq: f32) -> Self {
        let cos: Symbol<SAMPLES, RATE> =
            Symbol::new(&|t| (t * 2.0 * std::f32::consts::PI * freq).cos());

        let minus_cos = cos.apply_funtion(|x| x * -1.0);
        //let minus_cos: Symbol<SAMPLES, RATE> = cos.into_iter().map(|v| v * -1.0).collect();

        // Calc the energy for those sygnals is useless... the mean of the two will be alwasy zero

        Self {
            time: SignalPieceSlice::get_time(),
            symbols: vec![cos, minus_cos],
            symbol_period: SAMPLES as f32 * (1.0 / RATE as f32),
        }
    }
}

impl<const SAMPLES: usize, const RATE: usize, const NSYMBOLS: usize>
    ModDemod<SAMPLES, RATE, NSYMBOLS> for BPSK<SAMPLES, RATE, NSYMBOLS>
{
    fn bit_per_symbol(&self) -> u8 {
        1
    }

    // Also this could be reimplemented creating the time only once!
    fn time(&self) -> SignalPieceSlice<SAMPLES, RATE> {
        self.time.clone()
    }

    // Also this could be reimplemented creating the time only once!
    // TODO: decide with WHICH precision I want to work
    fn symbol_period(&self) -> f32 {
        self.symbol_period
    }

    fn symbols(&self) -> &[Symbol<SAMPLES, RATE>] {
        &self.symbols[..]
    }

    fn symbols_demodulation(&self, input: SignalPieceVec<RATE>) -> Result<Vec<usize>, DemodErr> {
        // based use for the calc of the integral (Reinmann)
        let base = self.symbol_period / SAMPLES as f32;

        // TODO: implement internal product between two signals
        let raw: Vec<f32> = SignalPieceVec::inner(input)
            .chunks(SAMPLES)
            .map(|raw_symbol| {
                raw_symbol
                    .iter()
                    .zip(self.symbols[0].clone().into_iter())
                    .map(|(a, b)| (a * b) * base)
                    .sum::<f32>()
            })
            .collect();

        let raw_bytes = raw
            .iter()
            .map(|r| if *r > 0.0 { 0usize } else { 1usize })
            .collect();

        Ok(raw_bytes)
    }
}
