// MAKE all of this general to the floating precision
struct BPSK {
    //step_by: f32,
    //freq: f32,
    symbol_period: f32,
    time: Vec<f32>,
    symbols: Vec<Vec<f32>>,
}

impl BPSK {
    fn new(step_by: f32, freq: f32, symbol_period: f32) -> Self {
        let time: Vec<f32> = (0..(symbol_period / step_by) as u32)
            .map(|x| x as f32 * step_by)
            .collect();

        //println!("time: {time:?}");

        let cos: Vec<f32> = time
            .iter()
            .map(|x| (x * 2.0 * std::f32::consts::PI * freq).cos())
            .collect();

        //println!("cos: {cos:?}");

        let minus_cos = cos.iter().map(|v| *v * -1.0).collect();

        // Calc the energy for those sygnals is useless... the mean of the two will be alwasy zero
        //let energy = |symbol: &Vec<f32>| symbol.iter().map(|v: &f32| v * v).sum::<f32>();

        Self {
            symbol_period,
            time,
            symbols: vec![cos, minus_cos],
        }
    }

    fn module(&self, input: &Vec<u8>) -> Vec<(f32, f32)> {
        // 0 => cos
        // 1 => -cos
        input
            .iter()
            .enumerate()
            .map(|(index, val)| {
                self.time
                    .clone()
                    .iter()
                    .zip(self.symbols[*val as usize].clone().iter())
                    .map(|(x, y)| (x + index as f32 * self.symbol_period, *y))
                    .collect()
            })
            .collect::<Vec<Vec<(f32, f32)>>>()
            .concat()
    }

    //First is the recevived bits, second returned is the costellation
    fn demodule(&self, input: Vec<f32>) -> (Vec<u8>, Vec<f32>) {
        let samples = self.time.len();
        // based use for the calc of the integral (Reinmann)
        let base = self.symbol_period / samples as f32;

        let raw: Vec<f32> = input
            .chunks(samples)
            .map(|raw_symbol| {
                raw_symbol
                    .iter()
                    .zip(self.symbols[0].iter())
                    .map(|(a, b)| (a * b) * base)
                    .sum::<f32>()
            })
            .collect();

        (
            raw.iter()
                .map(|r| if *r > 0.0 { 0u8 } else { 1u8 })
                .collect(),
            raw,
        )
    }
}
