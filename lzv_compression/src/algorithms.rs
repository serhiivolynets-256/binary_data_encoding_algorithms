use crate::dictionary::{Dictionary, DictionaryOptions};
use crate::error::LzvError;
use bit_stream::{BitStreamReader, BitStreamWriter};

#[derive(Debug)]
pub struct LzvHeader {
    pub excess: u8,
    pub word_length: u32,
    pub clear_on_overflow: bool,
}

impl LzvHeader {
    pub fn byte_length() -> usize {
        6
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.excess];
        bytes.append(&mut self.word_length.to_le_bytes().to_vec());
        bytes.push(self.clear_on_overflow as u8);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<LzvHeader, LzvError> {
        if bytes.len() != 6 {
            return Err(LzvError::InvalidInput);
        }

        let excess = bytes[0];
        let word_length = u32::from_le_bytes(
            bytes
                .get(1..5)
                .ok_or(LzvError::InvalidInput)?
                .try_into()
                .map_err(|_| LzvError::InvalidInput)?,
        );
        let clear_on_overflow = bytes[5] != 0;

        Ok(Self {
            excess,
            word_length,
            clear_on_overflow,
        })
    }
}

#[derive(Debug)]
pub(crate) struct EncodeStepData {
    big_i: u32,
    dict: Dictionary,
}

impl Default for EncodeStepData {
    fn default() -> Self {
        Self {
            big_i: 0,
            dict: Dictionary::default(),
        }
    }
}

pub struct LzvEncoder {
    pub(crate) step_data: Option<EncodeStepData>,
    pub(crate) output: BitStreamWriter,
}

impl LzvEncoder {
    pub fn new(output_file: &str) -> Result<Self, LzvError> {
        let mut output = BitStreamWriter::create(output_file)?;
        output.skip_bytes(LzvHeader::byte_length());

        Ok(Self {
            step_data: None,
            output,
        })
    }

    pub fn process(&mut self, input_sequence: &[u8]) -> Result<(), LzvError> {
        for c in input_sequence {
            if let Some(step_data) = &mut self.step_data {
                Self::encoding_step(&mut self.output, *c, step_data)?;
            } else {
                self.first_encoding_step(*c);
            }
        }

        Ok(())
    }

    pub fn finish(&mut self) -> Result<u8, LzvError> {
        if let Some(step_data) = &mut self.step_data {
            self.output.write_bit_sequence(
                &step_data.big_i.to_le_bytes(),
                step_data.dict.word_bit_length() as usize,
            )?;
        }
        let excess = self.output.excess();

        let header = LzvHeader {
            excess,
            word_length: 16,
            clear_on_overflow: false,
        };

        // self.output.finish()?;
        self.output
            .write_byte_sequence_unchecked_at(&header.to_bytes(), 0)?;

        Ok(excess)
    }

    fn first_encoding_step(&mut self, c: u8) {
        let mut step_data = EncodeStepData::default();
        step_data.big_i = step_data.dict.index_of_symbol(c);

        self.step_data = Some(step_data);
    }

    fn encoding_step(
        output: &mut BitStreamWriter,
        c: u8,
        step_data: &mut EncodeStepData,
    ) -> Result<(), LzvError> {
        let seq_i = step_data.dict.index_of_record(step_data.big_i, c);

        if let Some(seq_i) = seq_i {
            step_data.big_i = seq_i; // s = s || c;
        } else {
            output.write_bit_sequence(
                &step_data.big_i.to_le_bytes(),
                step_data.dict.word_bit_length() as usize,
            )?;

            step_data.dict.add(c, step_data.big_i);
            step_data.big_i = c as u32;
            step_data.big_i = step_data.dict.index_of_symbol(c);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DecodeStepData {
    old_i: u32,
    s: Vec<u8>,
}

impl Default for DecodeStepData {
    fn default() -> Self {
        Self {
            old_i: 0,
            s: vec![],
        }
    }
}

#[derive(Debug)]
pub struct LzvDecoder {
    pub(crate) step_data: Option<DecodeStepData>,
    pub(crate) input: BitStreamReader,
    pub(crate) output: BitStreamWriter,
    pub(crate) dict: Dictionary,
}

impl LzvDecoder {
    pub fn new(input_file: &str, output_file: &str) -> Result<Self, LzvError> {
        let mut input = BitStreamReader::open(input_file)?;

        let header_bytes = input.read_bit_sequence(LzvHeader::byte_length() * 8)?;
        let header = LzvHeader::from_bytes(&header_bytes)?;
        let dict = Dictionary::with_options(DictionaryOptions::new(
            header.word_length,
            header.clear_on_overflow,
        ));

        let output = BitStreamWriter::create(output_file)?;

        Ok(Self {
            step_data: None,
            input,
            output,
            dict,
        })
    }

    fn fit_ful_u32_if_need(input: &mut Vec<u8>) -> Result<(), LzvError> {
        if input.is_empty() {
            return Err(LzvError::InvalidInput);
        } else {
            while input.len() < 4 {
                input.push(0);
            }
        }

        Ok(())
    }

    pub fn process_all(&mut self) -> Result<(), LzvError> {
        loop {
            let mut bytes = self
                .input
                .read_bit_sequence(self.dict.word_bit_length() as usize)?;

            if Self::fit_ful_u32_if_need(&mut bytes).is_err() {
                return Ok(());
            };

            let index = if let Ok(arr) = <&[u8; 4]>::try_from(&*bytes) {
                u32::from_le_bytes(*arr)
            } else {
                return Err(LzvError::InvalidInput);
            };

            self.process(index)?;
        }
    }

    pub fn process(&mut self, index: u32) -> Result<(), LzvError> {
        if let Some(step_data) = &mut self.step_data {
            Self::decoding_step(
                &mut self.input,
                &mut self.output,
                index,
                step_data,
                &mut self.dict,
            )?;
        } else {
            self.step_data = Some(Self::first_decoding_step(&mut self.output, index)?);
        }

        Ok(())
    }

    fn first_decoding_step(
        output: &mut BitStreamWriter,
        index: u32,
    ) -> Result<DecodeStepData, LzvError> {
        if index > u8::MAX as u32 {
            return Err(LzvError::InvalidFirstIndex { index });
        }

        let mut step_data = DecodeStepData::default();
        let s = index as u8;

        output.write_bit_sequence(&[s], 8)?;

        step_data.old_i = index;
        step_data.s = vec![s];

        Ok(step_data)
    }

    fn decoding_step(
        input: &mut BitStreamReader,
        output: &mut BitStreamWriter,
        index: u32,
        step_data: &mut DecodeStepData,
        dict: &mut Dictionary,
    ) -> Result<(), LzvError> {
        let s_phrase = dict.get_record(index);

        let out_string = if let Some(s_phrase) = &s_phrase {
            s_phrase.clone()
        } else {
            let mut tmp = dict
                .get_record(step_data.old_i)
                .ok_or(LzvError::InvalidInput)?;
            tmp.push(step_data.s[0]);

            tmp
        };

        output.write_bit_sequence(&out_string, out_string.len() * 8)?;
        step_data.s = vec![out_string[0]];
        dict.add(out_string[0], step_data.old_i);
        step_data.old_i = index;

        Ok(())
    }
}
