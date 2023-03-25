use super::*;

// TODO: make sense implement an iterator over the signal?
#[derive(Clone)]
pub struct SignalPieceVec<const RATE: usize>(Vec<f32>);

impl<const RATE: usize> SignalPieceVec<RATE> {
    pub fn new(fun: &impl Fn(f32) -> f32, samples: usize) -> Self {
        let mut data = Vec::<f32>::new();

        Time::apply_over_time(samples, RATE, |_, v| data.push(fun(v)));
        Self(data)
    }

    pub fn get_time_with_samples(samples: usize) -> SignalPieceVec<RATE> {
        Self::new(&|t| t, samples)
    }

    pub fn get_time(&self) -> SignalPieceVec<RATE> {
        Self::get_time_with_samples(self.0.len())
    }

    // This method will move out what's inside of the object
    pub fn inner(val: Self) -> Vec<f32> {
        // this should move without copy
        val.0
    }

    // This method will move out what's inside of the object
    pub fn inner_ref(val: &Self) -> &Vec<f32> {
        // this should move without copy
        &val.0
    }

    // Apply a closure to every value in the array returning a new one
    pub fn apply_function(&mut self, fun: impl Fn(&mut f32)) {
        self.0.iter_mut().for_each(|i| fun(i));
    }

    // [f32; 2] is used to rapresent a point because egui use this representation
    // TODO: YOOO pls you have to work better with float precisions
    pub fn get_coordinates(&self, n_symbols: Option<(usize, usize)>) -> Vec<[f64; 2]> {
        let coordinates = self.get_time().into_iter().zip(self.0.iter());
        match n_symbols {
            Some((n, samples)) => coordinates
                .take(n * samples)
                .map(|(x, y)| [x as f64, *y as f64])
                .collect(),
            None => coordinates.map(|(x, y)| [x as f64, *y as f64]).collect(),
        }
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
            .for_each(|slices| res.extend(SignalPieceSlice::inner(slices.clone()).iter()));
        Self(res)
    }
}

impl<const RATE: usize> IntoIterator for SignalPieceVec<RATE> {
    type Item = f32;
    type IntoIter = <Vec<f32> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

impl<const RATE: usize> From<Vec<f32>> for SignalPieceVec<RATE> {
    fn from(input: Vec<f32>) -> Self {
        Self(input)
    }
}
