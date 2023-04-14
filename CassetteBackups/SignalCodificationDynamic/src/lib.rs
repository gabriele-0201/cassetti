pub mod traits;

// TODO: redesign of the errors
#[derive(Debug)]
pub enum ModErr {
    InvalidInput,
}

#[derive(Debug)]
pub enum DemodErr {
    InvalidInput,
    SyncNotFound,
    // the initial number specified a number of bytes
    // that is bigger than what the signal could contain
    SmallerThanExpected,
    Other(String),
}

// This struct will be used to iterate over the symbols
// that represent some bytes
//
pub struct RawSymbolIterator<'a> {
    n_bit: u8,
    // this will be every iteration the offset on wich apply the mask
    starting_bit: u8,
    remaining: &'a [u8],
}

/// Struct used to unpack symbols from raw bytes
pub struct RawSymbols<'a> {
    n_bit: u8,
    remaining: &'a [u8],
}

impl<'a> RawSymbols<'a> {
    /// This will return a Simple wrapper that is able to parse the bytes
    /// symbol after symbol
    ///
    /// If the number of bits make impossible to cover perfectly all the bytes
    /// than it will fail
    fn try_get_symbols(vec: &'a Vec<u8>, n_bit: u8) -> Result<Self, &'static str> {
        if (vec.len() * 8) % n_bit as usize != 0 {
            return Err("Impossible extract perfect symbols from here, maybe is possible but now I'm too tired to think");
        }
        Ok(Self {
            n_bit,
            remaining: &vec[..],
        })
    }

    /// Convert an interator over RawSymbols to a Vec<f32> representing the Signal
    /// formed using the symbols specified by the modulation
    ///
    /// This function CAN panic, if the symbol in the iter is bigger
    /// than the specified symbols
    fn to_signal_vec(self, symbols: &[traits::Symbol]) -> Vec<f32> {
        self.into_iter()
            .map(|n_symbol| symbols[n_symbol].clone())
            .collect::<Vec<Vec<f32>>>()
            .concat()
    }
}

impl<'a> IntoIterator for RawSymbols<'a> {
    type Item = usize;
    type IntoIter = RawSymbolIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RawSymbolIterator {
            n_bit: self.n_bit,
            starting_bit: 7,
            remaining: self.remaining,
        }
    }
}

impl<'a> Iterator for RawSymbolIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.len() == 0 {
            return None;
        }

        // holder index will never be bigger than 256
        // and a resonable amount of symbols to send is 2^32
        // so the holder will manage at most this number of symbols
        let (mut byte_index, mut arr_index, mut holder): (u8, usize, u32) =
            (self.starting_bit, 0, 0);

        // Here I can directly access to the array
        // because I already check that symbols fit perfectly
        for holder_index in (0..self.n_bit).rev() {
            let b = 1 << byte_index;

            if self.remaining[arr_index] & b == b {
                holder = holder | (1 << holder_index);
            }

            byte_index = match byte_index.checked_sub(1) {
                None => {
                    arr_index += 1;
                    7
                }
                Some(v) => v,
            };
        }

        self.starting_bit = byte_index as u8;
        self.remaining = &self.remaining[arr_index..];

        Some(holder as usize)
    }
}

/* THIS approach seems to be impossibel because NBIT in IntoIterator implementation
 * is NOT constrained

pub struct RawSymbolIterator<'a, const NBIT: usize> {
    //n_bit: usize,
    starting_bit: u8,
    remaining: &'a [u8],
}

impl<'a, const NBIT: usize> IntoIterator for &'a Vec<u8> {
    type Item = usize;
    type IntoIter = RawSymbolIterator<'a, NBIT>;

    fn into_iter(self) -> Self::IntoIter {
        RawSymbolIterator::<NBIT> {
            //n_bit: self.n_bit,
            starting_bit: 0,
            remaining: self,
        }
    }
}
*/

/// Struct used to pach bytes from raw symbols
/*
pub struct RawBytes<'a> {
    n_bit: u8,
    remaining: &'a [u8],
}
*/

// TODO here could be implemented FromIter so that
// we could wrap a Vec with RawBytes and with .collect() convert it
// to properly bytes based on the n_bit specified
//
// For now I will do a simply method

// TODO: I do not like this
// TODO: update the return type to DemodErr
pub fn collect_bytes_from_raw_bytes(
    raw_bytes: Vec<usize>,
    n_bit: u8,
) -> Result<Vec<u8>, &'static str> {
    if raw_bytes.is_empty() || n_bit == 0 {
        return Err(
            "You tried to convert an empty vec of signals or used 0 as number of bit per signal",
        );
    }

    let mut bytes = Vec::<u8>::new();

    let mut push_and_get_last_byte = || -> &mut u8 {
        // Here I go unsafe because I want to make multiple mutable borrows of bytes
        // This is totally safe becase I will every time get a mutable reference of ne next bytes
        // and drop the previous mutable reference
        //
        // This seems to be impossible in an unsafe way because while I have a mutable reference
        // to the last byte of bytes I can't add a new one to bytes and get now a mutable reference to the
        // current last because there is still the previous reference alive

        bytes.push(0);
        let ref_last: &u8 = &bytes[bytes.len() - 1];

        unsafe { &mut *(ref_last as *const u8 as *mut u8) }
    };
    let mut current_byte: &mut u8 = push_and_get_last_byte();

    // symbol_index, byte_index, bytes_byte_index
    let (mut symbol_i, mut byte_i): (u8, u8) = (n_bit - 1, 7);

    // crete a mask inside an usize
    // given the boundaries (included in the mask)
    let get_mask = |from: u8, to: u8| -> usize {
        let mut mask = 0;
        for i in from..=to {
            mask = mask | 1 << i;
        }
        mask
    };

    let mut place_bits = |symbol: usize| -> Option<()> {
        // symbol_index_low
        let sil = match symbol_i.checked_sub(byte_i) {
            Some(val) => val,
            None => 0,
        };
        let mask: usize = get_mask(sil, symbol_i);
        let bits_len: u8 = symbol_i - sil + 1;
        let offset: u8 = byte_i + 1 - bits_len; // maybe here the plus one should be removed

        let bits: u8 = (((symbol & mask) >> sil) as u8) << offset; // 0000 1000
        *current_byte |= bits;

        byte_i = match byte_i.checked_sub(bits_len) {
            Some(val) => val,
            None => {
                current_byte = push_and_get_last_byte();
                7
            }
        };

        symbol_i = match symbol_i.checked_sub(bits_len) {
            Some(val) => val,
            None => n_bit - 1,
        };

        if symbol_i == n_bit - 1 {
            return None;
        }

        return Some(());
    };

    let symbol_mask = get_mask(0, n_bit - 1);

    for symbol in raw_bytes {
        // First a mask will be used to use only the first n_bit inside the symbol
        if symbol & symbol_mask != symbol {
            return Err("You are trying to convert and incorrect symbol");
        }

        while let Some(()) = place_bits(symbol) {}
    }

    // When the byte is finished place_bits will alwasy add a new byte
    // so now I will remove the last one

    bytes.pop();

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpack_1_bit() {
        let vec = vec![1, 2]; // 0000 0001 0000 0010
        let raw_symbols = RawSymbols::try_get_symbols(&vec, 1).unwrap();
        assert_eq!(
            vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0],
            raw_symbols.into_iter().collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_unpack_2_bit() {
        let vec = vec![6, 3]; // 0000 0110 0000 0011
        let raw_symbols = RawSymbols::try_get_symbols(&vec, 2).unwrap();
        assert_eq!(
            vec![0, 0, 1, 2, 0, 0, 0, 3],
            raw_symbols.into_iter().collect::<Vec<usize>>()
        );

        let vec: Vec<u8> = vec![39, 141]; // 0010 0111 1000 1101
        let raw_symbols = RawSymbols::try_get_symbols(&vec, 2).unwrap();
        assert_eq!(
            vec![0, 2, 1, 3, 2, 0, 3, 1],
            raw_symbols.into_iter().collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_unpack_3_bit() {
        let vec = vec![6, 3, 3]; // 000|0 01|10 0|000| 001|1 00|00 0|011
        let raw_symbols = RawSymbols::try_get_symbols(&vec, 3).unwrap();
        assert_eq!(
            vec![0, 1, 4, 0, 1, 4, 0, 3],
            raw_symbols.into_iter().collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_unpack_4_bit() {
        let vec = vec![5, 12]; // 0000 0101 0011 1001
        let raw_symbols = RawSymbols::try_get_symbols(&vec, 4).unwrap();
        assert_eq!(
            vec![0, 5, 0, 12],
            raw_symbols.into_iter().collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_unpack_6_bit() {
        let vec = vec![1, 2, 4, 8, 16, 32]; // 0000 00|01 0000| 0010 00|00 0100| 0000 10|00 0001| 0000 00|10 0000
        let raw_symbols = RawSymbols::try_get_symbols(&vec, 6).unwrap();
        assert_eq!(
            vec![0, 16, 8, 4, 2, 1, 0, 32],
            raw_symbols.into_iter().collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_8_bit() {
        let vec = vec![123, 240];
        let raw_symbols = RawSymbols::try_get_symbols(&vec, 8).unwrap();
        assert_eq!(
            vec![123, 240],
            raw_symbols.into_iter().collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_unpack_12_bit() {
        let vec = vec![8, 16, 32]; // 0000 1000 0001 | 0000 0010 0000
        let raw_symbols = RawSymbols::try_get_symbols(&vec, 12).unwrap();
        assert_eq!(
            vec![129, 32],
            raw_symbols.into_iter().collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_pack_1_bit() {
        let vec = vec![0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1];
        assert_eq!(Ok(vec![85u8, 131u8]), collect_bytes_from_raw_bytes(vec, 1));
    }

    #[test]
    fn test_pack_2_bit() {
        let vec = vec![0, 1, 2, 3, 1, 0, 1, 2];
        assert_eq!(Ok(vec![27u8, 70u8]), collect_bytes_from_raw_bytes(vec, 2));
    }

    #[test]
    fn test_pack_3_bit() {
        let vec = vec![0, 1, 4, 0, 1, 4, 0, 3];
        assert_eq!(Ok(vec![6, 3, 3]), collect_bytes_from_raw_bytes(vec, 3));
    }

    #[test]
    fn test_pack_8_bit() {
        let vec = vec![255, 123, 23, 67, 98];
        assert_eq!(
            Ok(vec![255u8, 123u8, 23u8, 67u8, 98u8]),
            collect_bytes_from_raw_bytes(vec, 8)
        );
    }

    #[test]
    fn test_pack_12_bit() {
        let vec = vec![129, 32];
        assert_eq!(
            Ok(vec![8u8, 16u8, 32u8]),
            collect_bytes_from_raw_bytes(vec, 12)
        );
    }
}
