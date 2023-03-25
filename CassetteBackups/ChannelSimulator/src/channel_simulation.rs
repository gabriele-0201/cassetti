use Dsp::SignalDynamic::Signal;

pub fn add_noise_awgn(signal: &mut Signal, variance: f32) {
    use rand_distr::{Distribution, Normal};
    let normal = Normal::new(0.0, variance.sqrt()).unwrap();

    signal.apply_function(|v| *v += normal.sample(&mut rand::thread_rng()));
}

pub fn add_delay(signal: &mut Signal, delay: f32) {
    let rate = signal.rate();
    let step_by = 1.0 / rate as f32;
    let samples_delay: usize = dbg!((delay / step_by) as usize);
    let signal_vec = signal.inner_ref_mut();
    *signal_vec = [vec![0.0; samples_delay], signal_vec.clone()].concat();
}

pub fn add_additional_samples(signal: &mut Signal, additional_end_time: f32 /*seconds*/) {
    let samples_to_add = (additional_end_time * signal.rate() as f32) as usize;
    let signal_vec = signal.inner_ref_mut();
    *signal_vec = [signal_vec.clone(), vec![0.0; samples_to_add]].concat();
}
