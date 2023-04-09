use Dsp::SignalDynamic::Signal;
use SignalCodificationDynamic::{traits::*, *};

// Use this just to make the code more clear
const NSYMBOLS: usize = 2;

// MAKE all of this general to the floating precision
pub struct BPSK {
    //time: SignalPieceSlice<SAMPLES, RATE>,
    rate: usize,
    samples_per_symbol: usize,
    symbols: Vec<Symbol>,
    symbol_period: f32,
    sync: Vec<f32>,
    //sync_symbols: Vec<u8>,
    acceptance_sync_distance: f32,
    htx_energy: f32,
    use_expected_bytes: bool,
}

impl BPSK {
    // sync_symbols is a vec representing the symbols that will be used in the sync signal
    // here will be acceped bool just because BPSK has only two signals
    pub fn new(
        freq: f32,
        symbol_period: f32,
        rate: usize,
        sync_symbols: Vec<bool>,
        acceptance_sync_distance: f32,
        use_expected_bytes: bool,
    ) -> Self {
        // TODO: this should be checked
        let samples_per_symbol = (symbol_period * rate as f32) as usize;

        // here Htx is a rect(t/symbol_period)

        let cos: Symbol = Signal::new(
            &|t| (t * 2.0 * std::f32::consts::PI * freq).cos(),
            rate,
            samples_per_symbol,
        )
        .inner();

        // TODO: to update with htx.energy() when a signal as Htx will be accepted
        let htx_energy = 1.;

        let minus_cos = cos.iter().map(|x| x * -1.0).collect();

        let symbols = vec![cos, minus_cos];

        // TODO: why here I'm returing a tuple?
        let (_sync_symbols, sync): (Vec<u8>, Vec<Vec<f32>>) = sync_symbols
            .into_iter()
            .map(|symbol| {
                let symbol: usize = if symbol { 1 } else { 0 };
                (symbol as u8, symbols[symbol].clone())
            })
            .unzip();
        let sync = sync.concat();

        Self {
            rate,
            samples_per_symbol,
            symbols,
            sync,
            //sync_symbols,
            symbol_period,
            //symbol_period: samples as f32 * (1.0 / rate as f32),
            acceptance_sync_distance,
            htx_energy,
            use_expected_bytes,
        }
    }
}

impl ModDemod for BPSK {
    fn bit_per_symbol(&self) -> u8 {
        1
    }

    fn rate(&self) -> usize {
        self.rate
    }

    fn samples_per_symbol(&self) -> usize {
        self.samples_per_symbol
    }

    fn symbols(&self) -> &[Symbol] {
        &self.symbols[..]
    }

    fn use_expected_bytes(&self) -> bool {
        self.use_expected_bytes
    }

    fn get_average_symbols_energy(&self) -> f32 {
        self.htx_energy / 2.
    }

    fn get_sync(&self) -> Vec<f32> {
        self.sync.clone()
    }

    fn sync(&self, input: &mut Signal) -> Result<(), DemodErr> {
        if self.sync.is_empty() {
            return Ok(());
        }

        let signal = input.inner_ref_mut();

        // FIRST IMPLEMENTATION -> MUST BE CLEVER
        // Iterate over windows of the signal (big as sync signal)
        // than iterate over every symbol inside the possible sync signal
        // if every symbol match than BOOM
        //
        // If there is an error inside the sync than FOR now I will return an errror
        // later I will develop something that will decide the best possible
        /*
        for (i_sync, p_sync) in signal.windows(self.sync.len()).enumerate() {
            let mut found = true;
            for (i, p_symbol) in p_sync.chunks(symbol_len).enumerate() {
                let internal_product_res = Signal::from_vec(p_symbol.to_vec(), self.rate)
                    .internal_product(symbol_signal.clone());

                let found_symbol = if internal_product_res > 0.0 { 0u8 } else { 1u8 };

                if self.sync_symbols[i] != found_symbol {
                    found = false;
                    break;
                }
            }
            if found {
                signal.drain(0..dbg!(dbg!(i_sync) + self.sync.len()));
                return Ok(());
            }
        }
        */

        // SECOND IMPLEMENTATION
        // I will iterate over all the possible windows in the first DELTA of the signal
        // and the sync point will be the maximum of the derivative of the error (maybe better root mean sqare error)
        //
        // IDEA:
        // non toccare piu' i simboli gia' demodulati, bisogna fare una derivata del risultato del prodotto interno!!
        // sara' quel coefficente a migliorare e potrei prendere semplicamente quello MIGLIORE, quindi quello che ha una media
        // rispetto a tutti i simboli presenti nel sync piu' alta

        // THIRD IMPLEMENTATION
        // EASIEST ONE -> internal product of EVERY windows from the beginning, if the distance is lass then
        // acceptance_delta than this is the sync symbol
        /*
        let mut prev_distance: f32 = 0.;
        // TODO: this depend on the RATE
        let mut entered_acceptance = false;
        let mut counter_bigger = 0usize;

        let sync_signal: Signal = (self.get_sync(), self.rate).into();
        let sync_len = sync_signal.inner_ref().len();
        let sync_energy: f32 = /*dbg!(*/sync_signal.energy()/*)*/;

        let bigger_windows_requird = 10;

        let distance = |a: f32, b: f32| (a - b).abs();

        for (i_sync, p_sync) in signal
            .windows(self.sync.len())
            .enumerate()
            .take(sample_delta)
        {
            let internal_product = Signal::from_vec(p_sync.to_vec(), self.rate)
                .internal_product((self.get_sync(), self.rate).into());

            if distance(internal_product, sync_energy) <= self.acceptance_sync_distance {
                //signal.drain(0..dbg!(dbg!(i_sync) + sync_len));
                signal.drain(0..i_sync + sync_len);
                return Ok(());
            }

            /*
            if !entered_acceptance {
                if distance(internal_product, sync_energy) <= acceptance_delta {
                    entered_acceptance = true;
                    prev_distance = internal_product;
                }
            } else {
                match (
                    &mut prev_distance,
                    dbg!(distance(internal_product, sync_energy)),
                ) {
                    // the current is more near than the previous
                    (p_distance, distance) if distance <= *p_distance => {
                        println!("NEW MIN: {}", distance);
                        *p_distance = distance;
                    }
                    // if the new window is smaller than the bigger one found before
                    // than count a new bigger window
                    // it the bigger window execeed `bigger_windows_requird` than the sync signal is found
                    (_, _) => {
                        counter_bigger += 1;
                    }
                }
            }

            if counter_bigger >= bigger_windows_requird {
                signal.drain(0..dbg!(dbg!(i_sync) - dbg!(counter_bigger)));
                return Ok(());
            }
            */

        }
         */

        // FOURTH IMPLEMENTATION ->
        // eval distance of each window in the delta and take the minimum one

        let sync_signal = Signal::from_vec(self.get_sync(), self.rate);
        let sync_len = sync_signal.inner_ref().len();
        let sync_energy: f32 = sync_signal.energy();
        let distance = |vec: &[f32]| {
            let internal_product = sync_signal.internal_product((vec.to_vec(), self.rate).into());
            (internal_product - sync_energy).abs()
        };

        let mut window_iter = signal.windows(self.sync.len()).enumerate();

        // This is the percentage of the signal that will be used
        // to search for a sync signal
        const DELTA_MAX: f32 = 1.0;
        const DELTA_STEP: f32 = 0.05;
        let mut curr_delta = DELTA_STEP;
        let sample_delta = (DELTA_STEP * self.rate() as f32) as usize;

        //let (mut min_distance_index, min_distance) = (0, std::cell::RefCell::new(None));
        let (mut min_distance_index, mut min_distance) = (0, None);

        let mut found = false;

        while !found {
            loop
            /*window_iter.take_while(sample_delta)*/
            {
                let (i_sync, p_sync) = window_iter.next().expect("Should never go to the end");

                // RefCell is only used to avoid the next line:
                // min_distance_cloned = min_distance.clone();
                // is this usefull?? NO
                // is this funny?? YES
                /*
                let mut update = |new_d| {
                    *min_distance.clone().get_mut() = Some(new_d);
                    min_distance_index = i_sync;
                };
                match (*min_distance.try_borrow().unwrap(), distance(p_sync)) {
                    (None, d) => update(d),
                    (Some(m_d), d) if m_d > d => update(d),
                    _ => (),
                }
                */
                match (min_distance, distance(p_sync)) {
                    (None, d) => {
                        min_distance = Some(d);
                        min_distance_index = i_sync;
                    }
                    (Some(m_d), d) if m_d > d => {
                        min_distance = Some(d);
                        min_distance_index = i_sync;
                    }
                    _ => (),
                };

                if (i_sync + 1) % sample_delta == 0 {
                    break;
                }
            }
            // TODO
            // If the minum distance is below the acceptance delta THAN that's the stuff
            // if not than esaminate a new delta of the signal
            // if delta is biggger then max_denlta than => signal not found

            if matches!(min_distance, Some(m_d) if m_d < self.acceptance_sync_distance) {
                found = true;
            } else {
                if curr_delta >= DELTA_MAX {
                    return Err(DemodErr::SyncNotFound);
                }
                curr_delta += DELTA_STEP;
            }
        }

        signal.drain(0..min_distance_index + sync_len);
        Ok(())
    }

    fn symbols_demodulation(&self, mut input: Signal) -> Result<Vec<usize>, DemodErr> {
        // based use for the calc of the integral (Reinmann)

        //println!("len prev sync: {}", input.inner_ref().len());

        // SYNC the signal
        self.sync(&mut input)?;

        //println!("len after sync: {}", input.inner_ref().len());

        // NOT use iterator but use something like drain to consume the first 4 bytes

        let mut input = input.inner();
        let cos_mutliplier = (2.0 / self.htx_energy).sqrt();

        /* PRINT SOME TESTS STUFF
        println!("H_tx energy: {}", self.htx_energy); // should be equal to the symbol period

        // ensure that the energy of the symbols are +-(htx_energy / 2).sqrt() = +-0.7
        println!(
            "cos energy: {}",
            Signal::from_vec(self.symbols[0].to_vec(), self.rate).energy()
        );
        println!(
            "minus cos energy: {}",
            Signal::from_vec(self.symbols[1].to_vec(), self.rate).energy()
        );

        println!("cos multiplier: {}", cos_mutliplier);
        */

        let base = Signal::new_with_indeces(
            &|i, _t| self.symbols[0][i] * cos_mutliplier,
            self.rate,
            self.samples_per_symbol,
        );

        let mut symbols_to_take = input.len();

        // samples to take for the initial number
        if self.use_expected_bytes {
            let mut samples_to_take = (4.0 * 8.0 * self.samples_per_symbol as f32) as usize;

            let expected_num_samples: Vec<f32> = input.drain(..samples_to_take).collect();

            let mut num_expected_bytes = 0;
            let mut index = 7i8;

            for (index_b, raw_symbol) in expected_num_samples
                .chunks(self.samples_per_symbol)
                .enumerate()
            {
                num_expected_bytes |=
                    (if base.internal_product((raw_symbol.to_vec(), self.rate).into()) > 0.0 {
                        0
                    } else {
                        1
                    }) << (index as usize + index_b / 8);
                index = (index - 1).rem_euclid(8);
            }

            // sample to take to demodule the signal
            symbols_to_take = num_expected_bytes * 8;
            samples_to_take = symbols_to_take * self.samples_per_symbol;

            if samples_to_take > input.len() {
                return Err(DemodErr::SmallerThanExpected);
            }
        }

        let raw: Vec<f32> = input
            .chunks(self.samples_per_symbol)
            .take(symbols_to_take)
            .map(|raw_symbol| base.internal_product((raw_symbol.to_vec(), self.rate).into()))
            .collect();

        let raw_bytes = raw
            .iter()
            .map(|r| if *r > 0.0 { 0usize } else { 1usize })
            .collect();

        Ok(raw_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TO make the tests work DELTA_MAX must be 1.0
    // becase the delay is REALLY bigger than the real signal itself

    #[test]
    fn test_sync() {
        use rand_distr::{Distribution, Uniform};

        let rate = 1000;
        let freq: f32 = 100.0;
        let symbol_period: f32 = 1.0;

        // samples_per_symbol = symbol_period * rate
        // step_by = 1 / rate

        let uniform_symbols = Uniform::new(0, 2);
        let uniform_delay = Uniform::new(0, 1000);

        // number of symbols in the sync signal : [0..=]
        for n_symbols_sync in 1..=100 {
            println!("n symbols: {}", n_symbols_sync);
            for _ in 0..1 {
                // Generate a random sync sinal, will be used an uniform
                // distribution among symbols but the number of symbols is
                // specified at creation
                let sync_symbols: Vec<bool> = (0..n_symbols_sync)
                    .into_iter()
                    .map(|_| {
                        if uniform_symbols.sample(&mut rand::thread_rng()) == 0 {
                            false
                        } else {
                            true
                        }
                    })
                    .collect();

                let bpsk = BPSK::new(freq, symbol_period, rate, dbg!(sync_symbols), 0.001);

                //let bytes: Vec<u8> = vec![39, 141]; //0010 0111 1000 1101
                let bytes: Vec<u8> = vec![1]; //0000 0001

                let modulated_bytes = bpsk.module(&bytes).expect("IMP modulation");

                // Test over 10 possible random offset, from 0 to 10000 offset of samples
                for _/*delay_amount*/ in 0..5 {
                    let random_delay_amount = uniform_delay.sample(&mut rand::thread_rng());

                    let bytes_to_demodule = vec![
                        vec![0.0; dbg!(random_delay_amount)],
                        //vec![0.0; delay_amount],
                        modulated_bytes.clone().inner(),
                    ]
                    .concat();

                    let demod_bytes = bpsk
                        .demodule((bytes_to_demodule, rate).into())
                        .expect("IMP demodule");

                    //  001 0011 1100 0110|1
                    // 0001 0011 1100 0110

                    assert_eq!(bytes, demod_bytes);
                }
            }
        }
    }

    #[test]
    fn test_sync_with_noise() {
        use rand_distr::{Distribution, Normal, Uniform};

        let rate = 1000;
        let freq: f32 = 100.0;
        let symbol_period: f32 = 1.0;

        // samples_per_symbol = symbol_period * rate
        // step_by = 1 / rate

        let uniform_symbols = Uniform::new(0, 2);
        let uniform_delay = Uniform::new(0, 1000);

        // number of symbols always 10 for now
        let n_symbols_sync = 10;
        // noise variance divided by 10
        for noise_variance in 1..=20 {
            let normal_noise = Normal::new(0.0, noise_variance as f32 / 10.0).unwrap();
            for _ in 0..1 {
                // Generate a random sync sinal, will be used an uniform
                // distribution among symbols but the number of symbols is
                // specified at creation
                let sync_symbols: Vec<bool> = (0..n_symbols_sync)
                    .into_iter()
                    .map(|_| {
                        if uniform_symbols.sample(&mut rand::thread_rng()) == 0 {
                            false
                        } else {
                            true
                        }
                    })
                    .collect();

                let bpsk = BPSK::new(freq, symbol_period, rate, dbg!(sync_symbols), 0.001);

                //let bytes: Vec<u8> = vec![39, 141]; //0010 0111 1000 1101
                let bytes: Vec<u8> = vec![1]; //0000 0001

                let modulated_bytes = bpsk.module(&bytes).expect("IMP modulation");

                // Test over 10 possible random offset, from 0 to 10000 offset of samples
                for _/*delay_amount*/ in 0..1 {
                    let random_delay_amount = uniform_delay.sample(&mut rand::thread_rng());

                    let bytes_to_demodule = vec![
                        vec![normal_noise.sample(&mut rand::thread_rng()); dbg!(random_delay_amount)],
                        //vec![0.0; delay_amount],
                        modulated_bytes.clone().inner(),
                    ]
                    .concat();

                    let demod_bytes = bpsk
                        .demodule((bytes_to_demodule, rate).into())
                        .expect("IMP demodule");

                    assert_eq!(bytes, demod_bytes);
                }
            }
        }
    }

    #[test]
    fn test_bigger_signal() {
        let rate = 1000;
        let freq: f32 = 100.0;
        let symbol_period: f32 = 1.0;

        let sync_symbols = vec![];

        let bpsk = BPSK::new(freq, symbol_period, rate, dbg!(sync_symbols), 0.001);

        let bytes: Vec<u8> = vec![39, 141]; //0010 0111 1000 1101

        let modulated_bytes = bpsk.module(&bytes).expect("IMP modulation");

        let bytes_to_demodule = vec![
            //vec![0.0; delay_amount],
            modulated_bytes.clone().inner(),
            vec![0.0; 12432],
        ]
        .concat();

        let demod_bytes = bpsk
            .demodule((bytes_to_demodule, rate).into())
            .expect("IMP demodule");

        //  001 0011 1100 0110|1
        // 0001 0011 1100 0110

        assert_eq!(bytes, demod_bytes);
    }

    #[test]
    fn test_single_channel_simlation() {
        let rate = 44100;
        let freq: f32 = 3000.0;
        let symbol_period: f32 = 0.001;

        let delay_sample_amount = 35432;
        let addition_sample_amount = 12333;
        let noise_variance: f32 = 0.6;

        let sync_symbols = vec![true, false, true, true, false, true, false, true];

        let bpsk = BPSK::new(freq, symbol_period, rate, sync_symbols, 0.001);

        let bytes: Vec<u8> = vec![39, 141]; //0010 0111 1000 1101

        let modulated_bytes = bpsk.module(&bytes).expect("IMP modulation");

        let mut bytes_to_demodule = vec![
            vec![0.0; delay_sample_amount],
            modulated_bytes.clone().inner(),
            vec![0.0; addition_sample_amount],
        ]
        .concat();

        use rand_distr::{Distribution, Normal};
        let normal = Normal::new(0.0, noise_variance.sqrt()).unwrap();

        bytes_to_demodule
            .iter_mut()
            .for_each(|v| *v += normal.sample(&mut rand::thread_rng()));

        let demod_bytes = bpsk
            .demodule((bytes_to_demodule, rate).into())
            .expect("IMP demodule");

        //  001 0011 1100 0110|1
        // 0001 0011 1100 0110

        assert_eq!(bytes, demod_bytes);
    }
}
