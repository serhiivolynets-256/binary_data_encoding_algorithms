use tracing::trace;

pub const DEFAULT_DICTIONARY_SIZE: usize = 256;
pub const DEFAULT_WORD_LEN: usize = 8;

#[derive(Debug)]
pub struct DictionaryOptions {
    word_length: u32,
    clear_on_overflow: bool,
    index_limit: usize,
}

impl DictionaryOptions {
    pub fn new(word_length: u32, clear_on_overflow: bool) -> Self {
        let index_limit = 2usize.pow(word_length);
        Self {
            word_length,
            clear_on_overflow,
            index_limit,
        }
    }
}

impl Default for DictionaryOptions {
    fn default() -> Self {
        let word_length = 16;
        let index_limit = 2usize.pow(word_length);
        Self {
            word_length,
            clear_on_overflow: false,
            index_limit,
        }
    }
}

#[derive(Debug)]
pub struct Dictionary {
    // pub(crate) dict: HashMap<u32, (u8, u32)>,
    pub(crate) dict: Vec<(u8, u32)>,
    pub(crate) options: DictionaryOptions,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self {
            dict: Vec::default(),
            options: DictionaryOptions::default(),
        }
    }
}

impl Dictionary {
    pub fn with_options(options: DictionaryOptions) -> Self {
        Self {
            dict: Vec::default(),
            options,
        }
    }

    pub fn word_bit_length(&self) -> u32 {
        self.options.word_length
    }

    pub fn current_len(&self) -> usize {
        self.dict.len() + DEFAULT_DICTIONARY_SIZE
    }

    pub fn add(&mut self, char: u8, prev_pos: u32) {
        trace!(
            "Adding new record: {}: ({}, {})",
            self.dict.len(),
            char,
            prev_pos
        );
        if self.current_len() >= self.options.index_limit {
            if self.options.clear_on_overflow {
                self.dict = Vec::default();
            }
        } else {
            self.dict.push((char, prev_pos));
        }
    }

    pub fn index_of_symbol(&self, c: u8) -> u32 {
        c as u32
    }

    pub fn index_of_record(&self, s_prev: u32, c_char: u8) -> Option<u32> {
        self.dict
            .iter()
            .enumerate()
            .find(|(_, (c_, s_))| c_char.eq(c_) && (s_prev).eq(s_))
            .map(|(index, _)| (index + DEFAULT_DICTIONARY_SIZE) as u32)
    }

    pub fn get_record(&self, mut index: u32) -> Option<Vec<u8>> {
        let mut output = Vec::new();

        while index >= DEFAULT_DICTIONARY_SIZE as u32 {
            let Some((char, prev)) = self.dict.get(index as usize - DEFAULT_DICTIONARY_SIZE) else {
                return None;
            };

            output.push(*char);
            index = *prev;
        }

        output.push(index as u8);
        output.reverse();

        Some(output)
    }

    pub fn index_of(&self, s: Option<u32>, c: u8) -> Option<u32> {
        if let Some(s) = s {
            self.index_of_record(s, c)
        } else {
            Some(self.index_of_symbol(c))
        }
    }
}
