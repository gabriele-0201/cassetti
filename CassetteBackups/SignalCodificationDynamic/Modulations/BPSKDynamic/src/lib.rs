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
    htx: Signal,
    base: Signal,
    base_energy: f32,
    average_symbol_energy: f32,
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
        sync_symbols: Vec<usize>,
        acceptance_sync_distance: f32,
        use_expected_bytes: bool,
    ) -> Self {
        let samples_per_symbol = (symbol_period * rate as f32) as usize;

        // TODO: accept this in the method's arguments
        // 2.sqrt() just to normalize the energy
        // something is not correclty depend on the htx stuff
        //let htx = Signal::new(&|_| 2_f32.sqrt(), rate, samples_per_symbol);
        let htx = Signal::new(&|_| 1., rate, samples_per_symbol);
        let htx_energy = htx.energy();

        let cos: Symbol = Signal::new_with_indeces(
            &|i, t| htx.inner_ref()[i] * (t * 2.0 * std::f32::consts::PI * freq).cos(),
            rate,
            samples_per_symbol,
        )
        .inner();

        let minus_cos = cos.iter().map(|x| x * -1.0).collect();

        let symbols = vec![cos, minus_cos];

        let base_mutliplier = (2.0 / htx_energy).sqrt();
        let base = Signal::new_with_indeces(
            &|i, t| {
                base_mutliplier * htx.inner_ref()[i] * (t * 2.0 * std::f32::consts::PI * freq).cos()
            },
            rate,
            samples_per_symbol,
        );
        let base_energy = base.energy();
        let average_symbol_energy = htx_energy / 2.;

        println!("H_tx energy: {}", htx_energy); // should be equal to the symbol period
        println!("base multiplier: {}", base_mutliplier);
        println!("Base energy: {}, should be 1", base_energy); // should be equal to the symbol period

        // ensure that the energy of the symbols are +-(htx_energy / 2).sqrt() = +-0.7
        let cos_energy = Signal::from_vec(symbols[0].to_vec(), rate).energy();
        println!("cos energy: {}, should be Eh / 2", cos_energy);
        let min_cos_energy = Signal::from_vec(symbols[1].to_vec(), rate).energy();
        println!("minus cos energy: {}, should be Eh / 2", min_cos_energy);
        let real_averge_energy = (cos_energy + min_cos_energy) / 2.;

        println!(
            "Average symbols Energy (Es), real: {} thorical: {}",
            real_averge_energy, average_symbol_energy
        );

        // TODO: if the symbols specified in the sync is not correct than
        // this could panic, find a way to solve this!
        let sync: Vec<f32> = sync_symbols
            .into_iter()
            .map(|symbol| symbols[symbol].clone())
            .collect::<Vec<Vec<f32>>>()
            .concat();

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
            htx,
            use_expected_bytes,
            base,
            base_energy,
            average_symbol_energy,
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
        self.average_symbol_energy
    }

    fn get_sync_info(&self) -> SyncInfo {
        SyncInfo {
            // TODO: OMG this clone is REALLY REALLY ugly
            sync_signal_vec: self.sync.clone(),
            acceptance_sync_distance: self.acceptance_sync_distance,
        }
    }

    fn symbols_demodulation(&self, input: Signal) -> Result<Vec<usize>, DemodErr> {
        let raw_bytes: Vec<usize> = input
            .inner()
            .chunks(self.samples_per_symbol)
            .map(|raw_symbol| {
                if self
                    .base
                    .internal_product((raw_symbol.to_vec(), self.rate).into())
                    >= 0.0
                {
                    0usize
                } else {
                    1usize
                }
            })
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
                let sync_symbols: Vec<usize> = (0..n_symbols_sync)
                    .into_iter()
                    .map(|_| uniform_symbols.sample(&mut rand::thread_rng()))
                    .collect();

                let bpsk = BPSK::new(freq, symbol_period, rate, dbg!(sync_symbols), 0.001, false);

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
                let sync_symbols: Vec<usize> = (0..n_symbols_sync)
                    .into_iter()
                    .map(|_| uniform_symbols.sample(&mut rand::thread_rng()))
                    .collect();

                let bpsk = BPSK::new(freq, symbol_period, rate, dbg!(sync_symbols), 0.001, false);

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
        let rate = 100;
        let freq: f32 = 1.0;
        let symbol_period: f32 = 1.0;

        let sync_symbols = vec![];

        let bpsk = BPSK::new(freq, symbol_period, rate, sync_symbols, 0.1, true);

        let bytes: Vec<u8> = vec![39, 141]; //0010 0111 1000 1101

        let modulated_bytes = bpsk.module(&bytes).expect("IMP modulation");

        let bytes_to_demodule = vec![modulated_bytes.clone().inner(), vec![0.0; 1]].concat();

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

        let sync_symbols = vec![1, 1, 0, 0, 1, 0, 1, 0];

        let bpsk = BPSK::new(freq, symbol_period, rate, sync_symbols, 0.001, true);

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
