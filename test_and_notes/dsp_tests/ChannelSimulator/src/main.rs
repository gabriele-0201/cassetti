mod plot;
mod signals;

use plot::*;
use signals::*;

fn main() {
    // TODO: TOO MUCH CONSTANT A REFACTOR IS NEEDED

    const SYMBOL_PERIOD: f32 = 0.01; // 1sec

    const STEP_BY: f32 = 0.0001; // Step every 1c sec
    const RATE: usize = (1.0 / STEP_BY) as usize;

    const SAMPLES: usize = (SYMBOL_PERIOD / STEP_BY) as usize;

    const NOISE_VARIANCE: f32 = 0.5;

    //const FREQ: f32 = 1.0; //1Hz
    const NSYMBOLS: usize = 16;
    const N_SYMBOLS_TO_SEND: usize = 2;

    let mfsk = MFSK::<{ SAMPLES }, { RATE }, { NSYMBOLS }>::new(50.0, 5.0, |x, freq| {
        (x * 2.0 * std::f32::consts::PI * freq).cos()
    })
    .unwrap();

    let bits_sent = get_random_bits(mfsk.bit_per_symbol, N_SYMBOLS_TO_SEND);

    let (time, mut signal): (Vec<f32>, Vec<f32>) =
        mfsk.module(&bits_sent).unwrap().into_iter().unzip();

    plot(
        "MfskModulation",
        format!("bits: {bits_sent:?}, mfsk modulation").as_str(),
        time.clone(),
        signal.clone(),
        None,
        None,
    )
    .unwrap();

    add_noise(&mut signal, NOISE_VARIANCE);

    plot(
        "MfskModulationWithNoise",
        format!("bits: {bits_sent:?}, bpsk modulation").as_str(),
        time,
        signal.clone(),
        None,
        None,
    )
    .unwrap();

    /*
    let (bits_received, bits_in_costellation) = bpsk.demodule(signal);

    let n_bits = bits_in_costellation.len();
    plot(
        "Costellation",
        format!("Costellation").as_str(),
        (1..=bits_in_costellation.len())
            .into_iter()
            .map(|v| v as f32)
            .collect::<Vec<f32>>(),
        bits_in_costellation,
        None,
        None,
    )
    .unwrap();

    println!("Send:     {bits_sent:?}");
    println!("Reveived: {bits_received:?}");
    */
}

fn get_random_bits(bit_per_symbol: usize, n_symbols: usize) -> Vec<u8> {
    let n = bit_per_symbol * n_symbols;
    let mut res = Vec::new();
    (0..n)
        .into_iter()
        .for_each(|_| res.push(if rand::random() { 1 } else { 0 }));
    res
}

// Add Gaussian Noise to a signal
fn add_noise(input: &mut Vec<f32>, variance: f32) {
    use rand_distr::{Distribution, Normal};
    let normal = Normal::new(0.0, variance.sqrt()).unwrap();

    input
        .iter_mut()
        .for_each(|v| *v += normal.sample(&mut rand::thread_rng()))
}
