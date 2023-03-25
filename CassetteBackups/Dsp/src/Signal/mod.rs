pub mod signal_slice;
pub mod signal_vec;

use signal_slice::*;
use signal_vec::*;

// TODO: generalize Signal over Domain (Time or Freq)
pub struct Signal<const RATE: usize, F: Fn(f32) -> f32>(F);

impl<const RATE: usize, F: Fn(f32) -> f32> Signal<RATE, F> {
    fn new(fun: F) -> Self {
        Self(fun)
    }

    fn get_vec(&self, samples: usize) -> SignalPieceVec<RATE> {
        SignalPieceVec::<RATE>::new(&self.0, samples)
    }

    fn get_array<const SAMPLES: usize>(&self) -> SignalPieceSlice<SAMPLES, RATE> {
        SignalPieceSlice::<SAMPLES, RATE>::new(&self.0)
    }
}

// Here I did not used the generics for samples and rate otherwise
// if in the code will be used multiple Time with different SAMPLES and RATE
// this would produce a massive compilation output
struct Time;

impl Time {
    fn apply_over_time(samples: usize, rate: usize, mut fun: impl FnMut(usize, f32)) {
        let step_by = 1.0 / rate as f32;
        (0..samples)
            .into_iter()
            .enumerate()
            .for_each(|(index, v)| fun(index, v as f32 * step_by))
    }
}
