use super::*;

// TODO: can I change this to represent a general Signal?
// The problem is that a signal has not a determinate number of SAMPLES so
// we can't use static array but we should use vectors
/// RATE = sampling rate
#[repr(transparent)]
#[derive(Clone)]
pub struct SignalPieceSlice<const SAMPLES: usize, const RATE: usize>([f32; SAMPLES]);

impl<const SAMPLES: usize, const RATE: usize> SignalPieceSlice<SAMPLES, RATE> {
    pub fn new(fun: &impl Fn(f32) -> f32) -> Self {
        let mut data = [0.0; SAMPLES];

        Time::apply_over_time(SAMPLES, RATE, |index, v| data[index] = fun(v));
        Self(data)
    }

    pub fn get_time() -> SignalPieceSlice<SAMPLES, RATE> {
        Self::new(&|t| t)
    }

    // This method will move out what's inside of the object
    pub fn inner(val: Self) -> [f32; SAMPLES] {
        // this should move without copy
        val.0
    }

    // Apply a closure to every value in the array returning a new one
    pub fn apply_funtion(&self, fun: impl Fn(f32) -> f32) -> Self {
        core::array::from_fn(|i| fun(self.0[i])).into()
    }
}

impl<const SAMPLES: usize, const RATE: usize> IntoIterator for SignalPieceSlice<SAMPLES, RATE> {
    type Item = f32;
    type IntoIter = core::array::IntoIter<f32, SAMPLES>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const SAMPLES: usize, const RATE: usize> From<[f32; SAMPLES]>
    for SignalPieceSlice<SAMPLES, RATE>
{
    fn from(input: [f32; SAMPLES]) -> Self {
        Self(input)
    }
}
