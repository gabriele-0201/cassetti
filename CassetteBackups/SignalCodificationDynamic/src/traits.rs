use super::*;

use Dsp::SignalDynamic::Signal;

pub type Symbol = Vec<f32>;

pub trait ModDemod {
    // The number of bit per symbol
    // Should be never bigger than 256
    fn bit_per_symbol(&self) -> u8;

    // Rate of the modulation
    fn rate(&self) -> usize;

    // Number of samples per symbol
    fn samples_per_symbol(&self) -> usize;

    // Slice over the symbol used by the modulation
    fn symbols(&self) -> &[Symbol];

    // Every modulation could have some symbols to properly sync the signal
    //fn get_sync(&self) -> Vec<f32>;
    //fn get_acceptance_sync_distance(&self) -> f32;

    fn get_sync_info(&self) -> SyncInfo;

    // This method could be easily evaluated with other implemented
    // method in the trait but is computetionally complex
    // so should be better re-implement it
    fn get_average_symbols_energy(&self) -> f32 {
        let rate = self.rate();
        let symbols = self.symbols();

        symbols
            .iter()
            .map(|s| Signal::from_vec(s.to_vec(), rate).energy())
            .sum::<f32>()
            / symbols.len() as f32
    }

    // return the abs max value that will be contained in all
    // the possible symbols
    // This method could be easily overwritten due to the fact that
    // max value in symbols is something static and can be evaluated only one,
    // at the creation of the modulation
    fn max_value_in_symbols(&self) -> f32 {
        let mut max = 0.;

        self.symbols().into_iter().for_each(|symbol| {
            match symbol
                .iter()
                .max_by(|x, y| {
                    x.abs()
                        .partial_cmp(&y.abs())
                        .expect("Impossible comparison with NaN")
                })
                .expect("IMP find max")
            {
                new_max if *new_max > max => max = *new_max,
                _ => (),
            }
        });

        max
    }

    // this method will take a signal, find a sync at the beginning
    // remove so that the demodulation now can start from the beginning
    // of the modulated signal
    fn sync(&self, input: &mut Signal) -> Result<(), DemodErr> {
        // FIRST IMPLEMENTATION idea:
        // Iterate over windows of the signal (big as sync signal)
        // than iterate over every symbol inside the possible sync signal
        // if every symbol match than BOOM
        //
        // If there is an error inside the sync than FOR now I will return an errror
        // later I will develop something that will decide the best possible

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
        // (view history to see the implementation)

        // FOURTH IMPLEMENTATION ->
        // eval distance of each window in the delta and take the minimum one
        // continue with the new delta if the minimun distance is NOT under a specified trashold

        let sync_info = self.get_sync_info();

        if sync_info.sync_signal_vec.is_empty() {
            return Ok(());
        }

        let signal = input.inner_ref_mut();

        let sync_signal = Signal::from_vec(sync_info.sync_signal_vec, self.rate());
        let sync_len = sync_signal.inner_ref().len();
        let sync_energy: f32 = sync_signal.energy();
        let distance = |vec: &[f32]| {
            let internal_product = sync_signal.internal_product((vec.to_vec(), self.rate()).into());
            (internal_product - sync_energy).abs()
        };

        let mut window_iter = signal.windows(sync_signal.inner_ref().len()).enumerate();

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
            loop {
                let (i_sync, p_sync) = window_iter.next().expect("Should never go to the end");
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
            // If the minum distance is below the acceptance delta THAN that's the stuff
            // if not than esaminate a new delta of the signal
            // if delta is biggger then max_denlta than => signal not found

            if matches!(min_distance, Some(m_d) if m_d < sync_info.acceptance_sync_distance) {
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

    // this method specify if the modulation is using or not
    // the tecnique of inserting at the beginning of the input the number of
    // expected bytes that were sent
    fn use_expected_bytes(&self) -> bool;

    // The return is a SignalPieceVec with the same rate of th
    fn module(&self, input: &Vec<u8>) -> Result<Signal, ModErr> {
        let mut n_bytes_signal = vec![];
        let sync = self.get_sync_info().sync_signal_vec;

        // ADD at the beginnig an u32 to represent the amount of bytes that
        // later needs to be demodulated
        if self.use_expected_bytes() {
            // CONVETION: I will push the byte in a little endian order
            // so the first byte to appear in the signal is the least significant
            n_bytes_signal = RawSymbols::try_get_symbols(
                &(input.len() as u32).to_le_bytes().to_vec(),
                self.bit_per_symbol(),
            )
            .map_err(|_| ModErr::InvalidInput)?
            .to_signal_vec(self.symbols());
        }

        // We have N symbols and the approach is ONLY for now:
        // + we take the input and split it in groups on log2(NSYMBOLS)
        // + convert every group in an integer and use that as index in the symbol's array
        let raw_symbols = RawSymbols::try_get_symbols(&input, self.bit_per_symbol())
            .map_err(|_| ModErr::InvalidInput)?;

        // The access to the array should never panic because
        // raw_symbols is already entirely checked based on bit_per_symbol
        let modulated_signal = raw_symbols.to_signal_vec(self.symbols());

        // The contentaion is useless if
        Ok(Signal::from_vec(
            [sync, n_bytes_signal, modulated_signal].concat(),
            self.rate(),
        ))
    }

    fn symbols_demodulation(&self, input: Signal) -> Result<Vec<usize>, DemodErr>;

    // What this method does is:
    // 1. search the sync signal
    // 2. demodule the first 4bytes ((((4*8) / n_bit_per_symbol).ceil()) * sample_per_symbol) to the
    // the number of exepcted bytes to demodule
    // (return Err if the expected bytes is less than the avaiable signal)
    // 3. demodule the just defined amount of signal
    // 4. return the bytes
    fn demodule(&self, mut input: Signal) -> Result<Vec<u8>, DemodErr> {
        // SYNC the signal
        //println!("len pre sync: {}", input.inner_ref().len());
        self.sync(&mut input)?;
        //println!("len after sync: {}", input.inner_ref().len());

        let mut expected_bytes = None;

        // Decode Expected Bytes
        if self.use_expected_bytes() {
            let samples_to_take = ((32. / self.bit_per_symbol() as f32).ceil()
                * self.samples_per_symbol() as f32) as usize;

            let expected_num_samples: Vec<f32> =
                input.inner_ref_mut().drain(..samples_to_take).collect();

            let raw_num_symbols =
                self.symbols_demodulation((expected_num_samples, self.rate()).into())?;

            let num_bytes =
                collect_bytes_from_raw_bytes(raw_num_symbols, self.bit_per_symbol(), None)
                    .map_err(|err| DemodErr::Other(err.to_string()))?;

            expected_bytes = Some(dbg!(u32::from_le_bytes(num_bytes.try_into().unwrap())));

            let expected_samples = ((8 * expected_bytes.unwrap()) as f32
                / self.bit_per_symbol() as f32)
                .ceil() as usize
                * self.samples_per_symbol();

            if expected_samples > input.inner_ref().len() {
                return Err(DemodErr::SmallerThanExpected);
            }

            let _res = input.inner_ref_mut().split_off(expected_samples);
        }

        let raw_bytes = self.symbols_demodulation(input)?;

        // if the expected_bytes is NOT specified in the signal
        // THEN all the remaining signal after the possible signal
        // is treated as valid information
        collect_bytes_from_raw_bytes(raw_bytes, self.bit_per_symbol(), expected_bytes)
            .map_err(|err| DemodErr::Other(err.to_string()))
    }
}
