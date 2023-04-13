use Dsp::SignalDynamic::Signal;
use SignalCodificationDynamic::{traits::*, *};

// MAKE all of this general to the floating precision
pub struct MQAM {
    //time: SignalPieceSlice<SAMPLES, RATE>,
    n_symbols: usize,
    bit_per_symbol: u8,
    rate: usize,
    samples_per_symbol: usize,
    symbol_period: f32,
    // This is the signal used inside the QAM formula
    h_signal_energy: f32,
    coefficients: Vec<f32>,
    costellation_values: Vec<f32>,
    base_x: Signal,
    base_y: Signal,
    // Index of the symbol is the bit representation
    // SO I need to apply "mappatura di grey" keeping this in mind
    symbols: Vec<Symbol>,
    indeces_to_symbols: Vec<Vec<usize>>,
    use_expected_bytes: bool,
}

impl MQAM {
    pub fn new(
        freq: f32,
        symbol_period: f32,
        rate: usize,
        n_symbols: usize, /*, h: Vec<f32>*/
        use_expected_bytes: bool,
    ) -> Self {
        let samples_per_symbol = (symbol_period * rate as f32) as usize;

        // Ensure that M = n_symbols =  L^2
        let L = match (n_symbols as f32).sqrt() {
            val if val.ceil() == val && val != 0. => val.ceil() as u8,
            _ => panic!("Number of symbols for a MQAM must be an EVEN power of two"),
        };

        // This method can't panic because I already check that it must
        // be different than 0
        let bit_per_symbol: u8 = n_symbols.ilog2() as u8;

        // evaluate BEFORE the COS ans SIN function
        macro_rules! eval {
            ($($n: ident),*) => {$(
                let $n = Signal::new(
                    &|t| (t * 2.0 * std::f32::consts::PI * freq).$n(),
                    rate,
                    samples_per_symbol,
                )
                    .inner();
            )*};
        }
        eval!(cos, sin);

        // TODO: accept H as argument
        // for now Htx is just a step signal
        // Amplitude of the rect is 2 just to heve Es = 10
        let h_signal = Signal::new(&|_| 2_f32.sqrt(), rate, samples_per_symbol);
        let h_signal_energy = dbg!(h_signal.energy());
        let h_signal = h_signal.inner();

        // i8 should be enough (M <= 4098)
        // Compute all the coefficients required for the QAM
        let coefficients: Vec<f32> = (0..L).map(|l| (2 * l as i8 - L as i8 + 1) as f32).collect();

        // Computed L Grey series
        let next_gray_vec = |vec: Vec<u8>, index: u8| -> Vec<u8> {
            [
                vec.clone(),
                vec.iter().rev().map(|v| v + (2 << index)).collect(),
            ]
            .concat()
        };

        /* TODO: add Verbose feature
        println!("Coefficients: {:?}", coefficients);
        let bit_vec = |vec: &Vec<u8>| {
            println!("Gray vec");
            for v in vec {
                println!("{:6b}", v);
            }
        };
        */

        // Eval an array containing Grey coefficients
        let mut gray_vec = vec![0u8, 1u8];
        for i in 0..(L / 2) - 1 {
            gray_vec = next_gray_vec(gray_vec, i);
            //bit_vec(&gray_vec);
        }

        // max bit used for every value in the grey coefficients
        let bit_per_gray = L.ilog2();

        // Vec of all the aviable symbol of the modulation
        let mut symbols: Vec<Symbol> = vec![vec![]; n_symbols];

        // Matrix of the costellation
        // Each value contain the relative index of the symbol
        // index also mean represented stream of bits
        let mut indeces_to_symbols = vec![vec![0 as usize; L as usize]; L as usize];
        for i in (0..L).rev() {
            for j in 0..L {
                let signal = Signal::new_with_indeces(
                    &|t_index, _| {
                        coefficients[j as usize] * cos[t_index] * h_signal[t_index]
                            - coefficients[i as usize] * sin[t_index] * h_signal[t_index]
                    },
                    rate,
                    samples_per_symbol,
                );

                let index_symbol =
                    ((gray_vec[j as usize] << bit_per_gray) | gray_vec[i as usize]) as usize;

                //print!("{:6b} ", index_symbol);
                symbols[index_symbol] = signal.inner();
                indeces_to_symbols[(L - 1 - i) as usize][j as usize] = index_symbol;
            }
            //println!("");
        }

        //println!("{:?}", indeces_to_symbols);

        let h_multiplier = (2.0 / h_signal_energy).sqrt();

        let base_y = Signal::new_with_indeces(
            &|t_index, _t| h_multiplier * cos[t_index] * h_signal[t_index],
            rate,
            samples_per_symbol,
        );

        let base_x = Signal::new_with_indeces(
            &|t_index, _t| h_multiplier * sin[t_index] * h_signal[t_index],
            rate,
            samples_per_symbol,
        );

        // Save the result of the internal product with the bases
        let coeff_multiplier = (h_signal_energy / 2.).sqrt();
        let costellation_values: Vec<f32> =
            coefficients.iter().map(|c| c * coeff_multiplier).collect();

        Self {
            rate,
            samples_per_symbol,
            n_symbols,
            bit_per_symbol,
            symbol_period,
            coefficients,
            costellation_values,
            h_signal_energy,
            symbols,
            indeces_to_symbols,
            base_x,
            base_y,
            use_expected_bytes,
        }
    }
}

impl ModDemod for MQAM {
    fn bit_per_symbol(&self) -> u8 {
        self.bit_per_symbol
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

    fn get_sync(&self) -> Vec<f32> {
        println!("NO sync supported on QAM yet");
        vec![]
    }

    fn get_average_symbols_energy(&self) -> f32 {
        ((self.symbols.len() - 1) as f32 / 3.) * self.h_signal_energy
    }

    fn sync(&self, input: &mut Signal) -> Result<(), DemodErr> {
        println!("NO sync supported on QAM yet");
        Ok(())
    }

    fn use_expected_bytes(&self) -> bool {
        self.use_expected_bytes
    }

    fn symbols_demodulation(&self, input: Signal) -> Result<Vec<usize>, DemodErr> {
        // First 4 bytes need to be demodulated
        // HERE the padding is really a mess
        if self.use_expected_bytes {}

        // compute the internal product with the basis for each raw symbol
        let x_y_res: Vec<(f32, f32)> = input
            .inner()
            .chunks(self.samples_per_symbol)
            .map(|raw_symbols| {
                let raw_symbol_signal: Signal = (raw_symbols.to_vec(), self.rate).into();
                (
                    /*
                    dbg!(self.base_x.internal_product(raw_symbol_signal.clone())),
                    dbg!(self.base_y.internal_product(raw_symbol_signal)),
                    */
                    self.base_x.internal_product(raw_symbol_signal.clone()),
                    self.base_y.internal_product(raw_symbol_signal),
                )
            })
            .collect();

        // TODO: this can be a binary search or maybe something even better (constant time)
        let find_nearest = |val: &f32| -> usize {
            let (mut i_min, mut min) = (0, (val - self.costellation_values[0]).abs());
            for (i, c) in self
                .costellation_values
                .iter() /*.skip(1)*/
                .enumerate()
            {
                let poss_min = (val - c).abs();
                if poss_min < min {
                    i_min = i;
                    min = poss_min
                }
            }
            i_min
        };

        // find the nearest point in the costellation for each demodulated point
        let x_y_indeces: Vec<(usize, usize)> = x_y_res
            .iter()
            .map(|(x, y)| {
                // BAD linear search for the nearest value
                (find_nearest(x), find_nearest(y))
            })
            .collect();

        Ok(x_y_indeces
            .iter()
            .map(|(x, y)| self.indeces_to_symbols[*x][*y])
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_4qam() {
        let rate = 1000;
        let qam = MQAM::new(100.0, 1.0, rate, 4);
        let bytes: Vec<u8> = vec![39, 141]; // 0010 0111 1000 1101

        // MAP:
        // 10 11
        // 00 01

        let mod_res: Vec<f32> = [
            qam.symbols[0].clone(),
            qam.symbols[2].clone(),
            qam.symbols[1].clone(),
            qam.symbols[3].clone(),
            qam.symbols[2].clone(),
            qam.symbols[0].clone(),
            qam.symbols[3].clone(),
            qam.symbols[1].clone(),
        ]
        .concat();

        let real_mod_res: Vec<f32> = qam.module(&bytes).expect("Impossible to module").inner();
        assert_eq!(mod_res, real_mod_res);

        let demod_bytes = qam
            .demodule((mod_res, rate).into())
            .expect("Impossible demod");

        assert_eq!(bytes, demod_bytes);
    }

    #[test]
    fn new_16qam() {
        let rate = 1000;
        let qam = MQAM::new(100.0, 1.0, rate, 16);
        let bytes: Vec<u8> = vec![39, 141, 173, 63, 182]; // 0010 0111 1000 1101 1010 1101 0011 1111 1011 0110

        // MAP:
        // 10    110   1110   1010
        // 11    111   1111   1011
        //  1    101   1101   1001
        //  0    100   1100   1000

        let mod_res: Vec<f32> = [
            qam.symbols[2].clone(),
            qam.symbols[7].clone(),
            qam.symbols[8].clone(),
            qam.symbols[13].clone(),
            qam.symbols[10].clone(),
            qam.symbols[13].clone(),
            qam.symbols[3].clone(),
            qam.symbols[15].clone(),
            qam.symbols[11].clone(),
            qam.symbols[6].clone(),
        ]
        .concat();

        let real_mod_res: Vec<f32> = qam.module(&bytes).expect("Impossible to module").inner();
        assert_eq!(mod_res, real_mod_res);

        let demod_bytes = qam
            .demodule((mod_res, rate).into())
            .expect("Impossible demod");

        assert_eq!(bytes, demod_bytes);

        // try modulate file
        let in_file_name = "file_test.org";

        let f = std::fs::File::open(in_file_name)
            .map_err(|_| ())
            .expect("IMP open file");

        use std::io::Read;
        let bytes = std::io::BufReader::new(f)
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .expect("IMP read from file");

        let mod_res: Vec<f32> = qam.module(&bytes).expect("Impossible to module").inner();

        let demod_bytes = qam
            .demodule((mod_res, rate).into())
            .expect("Impossible demod");

        assert_eq!(bytes, demod_bytes);

        let f = std::fs::File::create("File test outut.org")
            .map_err(|_| ())
            .expect("IMP open file");

        use std::io::Write;
        std::io::BufWriter::new(f)
            .write(&bytes)
            .expect("IMP read from file");
    }

    #[test]
    fn new_64qam() {
        let rate = 1000;
        let qam = MQAM::new(100.0, 1.0, rate, 64);
        let bytes: Vec<u8> = vec![39, 141, 173]; // 0010 01|11 1000| 1101 10|10 1101

        // MAP:
        // 10    110   1110   1010
        // 11    111   1111   1011
        // 1    101   1101   1001
        // 0    100   1100   1000

        let mod_res: Vec<f32> = [
            qam.symbols[9].clone(),
            qam.symbols[56].clone(),
            qam.symbols[54].clone(),
            qam.symbols[45].clone(),
        ]
        .concat();

        let real_mod_res: Vec<f32> = qam.module(&bytes).expect("Impossible to module").inner();
        assert_eq!(mod_res, real_mod_res);

        let demod_bytes = qam
            .demodule((mod_res, rate).into())
            .expect("Impossible demod");

        assert_eq!(bytes, demod_bytes);
    }
}
