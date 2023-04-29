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

    pub fn get_time(&self) -> [f32; SAMPLES] {
        let mut res = [0.0; SAMPLES];
        // This piece of could should never panic,
        // the aray is big `SAMPLES` and the range is exactely the same
        Time::apply_over_time(SAMPLES, RATE, |index, v| res[index] = v);
        res
    }
}

impl<const SAMPLES: usize, const RATE: usize> IntoIterator for SignalPieceSlice<SAMPLES, RATE> {
    type Item = f32;
    type IntoIter = core::array::IntoIter<f32, SAMPLES>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// TODO: make sense implement an iterator over the signal?
pub struct SignalPieceVec<const RATE: usize>(Vec<f32>);

impl<const RATE: usize> SignalPieceVec<RATE> {
    fn new(fun: &impl Fn(f32) -> f32, samples: usize) -> Self {
        let mut data = Vec::<f32>::new();

        Time::apply_over_time(samples, RATE, |_, v| data.push(fun(v)));
        Self(data)
    }

    fn get_time(&self, samples: usize) -> Vec<f32> {
        let mut res = Vec::<f32>::new();
        // This piece of could should never panic,
        // the aray is big `SAMPLES` and the range is exactely the same
        Time::apply_over_time(samples, RATE, |_, v| res.push(v));
        res
    }
}

// Implementation for from_iter starting from an iterator over multiple SignalPieceSlice
impl<const SAMPLES: usize, const RATE: usize> FromIterator<SignalPieceSlice<SAMPLES, RATE>>
    for SignalPieceVec<RATE>
{
    // I automatically request IntoIter<Item = SignalPieceSlice<SAMPLES, RATE>>
    fn from_iter<I: IntoIterator<Item = SignalPieceSlice<SAMPLES, RATE>>>(iter: I) -> Self {
        let mut res = Vec::<f32>::new();
        // PROBLEM: Here the collect will create a NEW signal keeping the old slices
        // I could use like TAKE to move stuff maybe BUT i'm not sure if this could work
        // at the end here I'm coping stuff from the stack to the heap
        iter.into_iter()
            .for_each(|slices| res.extend(slices.0.clone().iter()));
        Self(res)
    }
}

/*
// Implementation for from_iter starting from an iterator over multiple SignalPieceSlice refernces
impl<const SAMPLES: usize, const RATE: usize, S> FromIterator<&SignalPieceSlice<SAMPLES, RATE>>
    for SignalPieceVec<RATE>
{
    // I automatically request IntoIter<Item = SignalPieceSlice<SAMPLES, RATE>>
    fn from_iter<I: IntoIterator<Item = &SignalPieceSlice<SAMPLES, RATE>>>(iter: I) -> Self {
        let mut res = Vec::<f32>::new();
        // PROBLEM: Here the collect will create a NEW signal keeping the old slices
        // I could use like TAKE to move stuff maybe BUT i'm not sure if this could work
        // at the end here I'm coping stuff from the stack to the heap
        iter.into_iter()
            .for_each(|slices| res.extend(slices.0.clone().iter()));
        Self(res)
    }
}
*/
