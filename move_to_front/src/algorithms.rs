const ALPHABET_SIZE: usize = 256;

#[derive(Debug, Clone)]
struct Alphabet {
    index_of_symbol: [usize; ALPHABET_SIZE], // u8 -> usize
    symbol_at_index: [u8; ALPHABET_SIZE],    // usize -> u8
}

impl Default for Alphabet {
    fn default() -> Self {
        let index_of_symbol: [usize; ALPHABET_SIZE] = std::array::from_fn(|i| i);
        let symbol_at_index: [u8; ALPHABET_SIZE] = std::array::from_fn(|i| i as u8);

        Self {
            index_of_symbol,
            symbol_at_index,
        }
    }
}

impl Alphabet {
    pub fn index_of(&mut self, symbol: u8) -> u8 {
        let index = self.index_of_symbol[symbol as usize] as u8;
        self.update_alpha(index, symbol);

        index
    }

    pub fn symbol_of(&mut self, index: u8) -> u8 {
        let symbol = self.symbol_at_index[index as usize];
        self.update_alpha(index, symbol);

        symbol
    }

    pub fn update_alpha(&mut self, index: u8, symbol: u8) {
        for i in (0..index as usize).rev() {
            self.index_of_symbol[self.symbol_at_index[i] as usize] += 1;
            self.symbol_at_index[i + 1] = self.symbol_at_index[i];
        }

        self.index_of_symbol[symbol as usize] = 0;
        self.symbol_at_index[0] = symbol;
    }
}

#[derive(Default, Debug, Clone)]
pub struct MoveToFrontEncoder {
    pub(crate) alphabet: Alphabet,
}

impl MoveToFrontEncoder {
    pub fn encode(&mut self, word: &[u8]) -> Vec<u8> {
        word.iter()
            .map(|symbol| self.alphabet.index_of(*symbol))
            .collect::<Vec<_>>()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MoveToFrontDecoder {
    pub(crate) alphabet: Alphabet,
}

impl MoveToFrontDecoder {
    pub fn decode(&mut self, code: &[u8]) -> Vec<u8> {
        code.iter()
            .map(|index| self.alphabet.symbol_of(*index))
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithms::{MoveToFrontDecoder, MoveToFrontEncoder};
    use utils::init_tracing;

    #[test]
    fn test_move_to_front_encoder() {
        init_tracing();

        for size in 10..100 {
            let bytes = std::iter::repeat_with(|| rand::random::<u8>())
                .take(size)
                .collect::<Vec<_>>();

            let mut encoder = MoveToFrontEncoder::default();
            let mut decoder = MoveToFrontDecoder::default();

            let code = encoder.encode(&bytes);
            let decoded_bytes = decoder.decode(&code);

            assert_eq!(bytes, decoded_bytes);
        }
    }

    #[test]
    fn test_move_to_front_encoder_for_parts() {
        init_tracing();
        for size in 10..100 {
            let mut bytes = std::iter::repeat_with(|| rand::random::<u8>())
                .take(size)
                .collect::<Vec<_>>();

            let mut encoder = MoveToFrontEncoder::default();
            let mut decoder = MoveToFrontDecoder::default();

            let mut code = Vec::new();
            let (chunks, remainder) = bytes.as_chunks::<16>();
            for chunk in chunks {
                code.append(&mut encoder.encode(chunk));
            }

            code.append(&mut encoder.encode(remainder));

            let decoded_bytes = decoder.decode(&code);

            assert_eq!(bytes, decoded_bytes);
        }
    }
}
