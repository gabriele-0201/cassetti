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

// TODO: make sense implement an iterator over the signal?
#[derive(Debug, Clone)]
pub struct Signal {
    values: Vec<f32>,
    rate: usize,
}

impl Signal {
    pub fn new(fun: &impl Fn(f32) -> f32, rate: usize, samples: usize) -> Self {
        let mut values = Vec::<f32>::new();

        Time::apply_over_time(samples, rate, |_, v| values.push(fun(v)));
        Self { values, rate }
    }

    pub fn new_with_indeces(fun: &impl Fn(usize, f32) -> f32, rate: usize, samples: usize) -> Self {
        let mut values = Vec::<f32>::new();

        Time::apply_over_time(samples, rate, |i, v| values.push(fun(i, v)));
        Self { values, rate }
    }

    pub fn rate(&self) -> usize {
        self.rate
    }

    pub fn from_vec(values: Vec<f32>, rate: usize) -> Self {
        Self { values, rate }
    }

    pub fn get_time_with_samples(samples: usize, rate: usize) -> Signal {
        // WHAT?!?!?!?!?! here was the opposite
        Self::new(&|t| t, rate, samples)
    }

    pub fn get_time(&self) -> Signal {
        Self::get_time_with_samples(self.values.len(), self.rate)
    }

    pub fn get_duration(&self) -> f32 {
        self.values.len() as f32 * (1.0 / self.rate as f32)
    }

    // This method will move out what's inside of the object
    pub fn inner(self) -> Vec<f32> {
        // this should move without copy
        self.values
    }

    // This method will move out what's inside of the object
    pub fn inner_ref(&self) -> &Vec<f32> {
        // this should move without copy
        &self.values
    }

    // This method will move out what's inside of the object
    pub fn inner_ref_mut(&mut self) -> &mut Vec<f32> {
        // this should move without copy
        &mut self.values
    }

    // Apply a closure to every value in the array returning a new one
    pub fn apply_function(&mut self, fun: impl Fn(&mut f32)) {
        self.values.iter_mut().for_each(|i| fun(i));
    }

    // [f32; 2] is used to rapresent a point because egui use this representation
    // TODO: YOOO pls you have to work better with float precisions
    pub fn get_coordinates(&self, n_symbols: Option<usize>) -> Vec<[f64; 2]> {
        let coordinates = self.get_time().into_iter().zip(self.values.iter());
        match n_symbols {
            Some(samples) => coordinates
                .take(samples)
                .map(|(x, y)| [x as f64, *y as f64])
                .collect(),
            None => coordinates.map(|(x, y)| [x as f64, *y as f64]).collect(),
        }
    }

    // TODO update with REFERENCE of Signal
    pub fn internal_product(&self, vec: Signal) -> f32 {
        let base = 1.0 / self.rate as f32;
        self.values
            .iter()
            .zip(vec.values.iter())
            .map(|(a, b)| (a * b))
            .sum::<f32>()
            * base
    }

    pub fn energy(&self) -> f32 {
        let base = 1.0 / self.rate as f32;
        self.values.iter().map(|v| v * v).sum::<f32>() * base
    }
}

impl IntoIterator for Signal {
    type Item = f32;
    type IntoIter = <Vec<f32> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl From<(Vec<f32>, usize)> for Signal {
    fn from(input: (Vec<f32>, usize)) -> Self {
        Self {
            values: input.0,
            rate: input.1,
        }
    }
}
