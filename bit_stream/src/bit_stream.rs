use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use tracing::trace;

#[derive(Debug)]
pub struct BitStreamWriter {
    pub file: File,
    pub bytes_written: usize,
    pub current_byte: u8,
    pub bit_count: usize,
}

impl BitStreamWriter {
    pub fn create(path: &str) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            file,
            bytes_written: 0,
            current_byte: 0u8,
            bit_count: 0,
        })
    }

    pub fn write_bit_sequence(&mut self, data: &[u8], bits_amount: usize) -> std::io::Result<()> {
        trace!("Write {bits_amount} bits of [{data:?}]");

        let to_take = bits_amount.min(data.len() * 8);

        let mut buffer = Vec::new();
        for i in 0..to_take {
            let src_byte = i / 8;
            let src_bit = i % 8;

            let bit = (data[src_byte] >> src_bit) & 1;

            if bit != 0 {
                self.current_byte |= 1 << self.bit_count;
            }

            self.bit_count = (self.bit_count + 1) % 8;

            if self.bit_count == 0 {
                buffer.push(self.current_byte);
                self.current_byte = 0;
            }
        }

        if self.bit_count != 0 {
            buffer.push(self.current_byte);
        }

        self.file.seek(SeekFrom::Start(self.bytes_written as u64))?;

        if self.bit_count % 8 == 0 {
            self.bytes_written += buffer.len();
        } else {
            self.bytes_written += buffer.len() - 1
        }

        self.file.write_all(&buffer)?;

        // self.bit_len += to_take / 8;

        Ok(())
    }

    pub fn write_byte_sequence_unchecked_at(
        &mut self,
        bytes: &[u8],
        start: u64,
    ) -> std::io::Result<()> {
        trace!("Write byte sequence unchecked: [{bytes:?}] at {start}");
        self.file.seek(SeekFrom::Start(start))?;
        self.file.write_all(&bytes)?;

        Ok(())
    }

    pub fn skip_bytes(&mut self, amount: usize) {
        self.bytes_written += amount;
    }

    pub fn finish(self) -> std::io::Result<()> {
        Ok(())
    }

    pub fn excess(&self) -> u8 {
        self.bit_count as u8 // value from 0 to 8
    }
}

#[derive(Debug)]
pub struct BitStreamReader {
    file: File,
    bit_len: usize,
    read_pos: usize,
}

impl BitStreamReader {
    pub fn open(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let bit_len = file.metadata()?.len() as usize * 8;
        Ok(Self {
            file,
            bit_len,
            read_pos: 0,
        })
    }

    pub fn read_bit_sequence(&mut self, bits_amount: usize) -> std::io::Result<Vec<u8>> {
        let approx_byte_start = self.read_pos / 8;
        let approx_byte_read = (bits_amount + 7) / 8;
        let mut file_buffer = vec![0u8; approx_byte_read];

        self.file.seek(SeekFrom::Start(approx_byte_start as u64))?;
        let bytes_read = self.file.read(&mut file_buffer)?;

        let available = self.bit_len.saturating_sub(self.read_pos);

        let to_read = bits_amount.min(available);
        let mut out = vec![0u8; (to_read + 7) / 8];
        for i in 0..to_read {
            let absolute_bit = self.read_pos + i;
            let absolute_byte = absolute_bit / 8;
            let absolute_bit_in_byte = absolute_bit % 8;

            let bit = (file_buffer[absolute_byte - approx_byte_start] >> absolute_bit_in_byte) & 1;

            if bit != 0 {
                out[i / 8] |= 1u8 << i % 8;
            }
        }

        self.read_pos += to_read;

        trace!("Read {bits_amount} bits. They are [{out:?}]");
        Ok(out)
    }

    pub fn read_all(path: &str) -> std::io::Result<Vec<u8>> {
        let mut reader = Self::open(path)?;
        let mut buf = Vec::new();
        reader.file.read_to_end(&mut buf)?;

        Ok(buf)
    }

    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn byte_len(&self) -> usize {
        (self.bit_len + 7) / 8
    }
}

pub struct BitReader<'a> {
    bytes: &'a [u8],
    current_byte: usize,
    current_bit: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            current_byte: 0,
            current_bit: 0,
        }
    }
}

impl<'a> Iterator for BitReader<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_byte >= self.bytes.len() {
            return None;
        }

        let byte = self.bytes[self.current_byte];
        let bit = (byte >> self.current_bit) & 1;

        self.current_bit += 1;
        if self.current_bit == 8 {
            self.current_bit = 0;
            self.current_byte += 1;
        }

        Some(bit == 1)
    }
}

#[cfg(test)]
mod tests {
    use crate::{BitStreamReader, BitStreamWriter};
    const TEST_FILE_NAME: &str = "test.bin";

    #[test]
    fn test_all() {
        //////////////////////////////////////////////
        // example from task
        //////////////////////////////////////////////
        let a1 = [0xE1, 0x01];
        let a2 = [0xEE, 0x00];

        let mut writer = BitStreamWriter::create(TEST_FILE_NAME).unwrap();
        writer.write_bit_sequence(&a1, 9).unwrap();
        writer.write_bit_sequence(&a2, 9).unwrap();

        let mut reader = BitStreamReader::open(TEST_FILE_NAME).unwrap();
        assert_eq!(reader.read_bit_sequence(11).unwrap(), [0xE1, 0x05],);
        assert_eq!(reader.read_bit_sequence(7).unwrap(), [0x3B],);

        //////////////////////////////////////////////
        // empty write
        //////////////////////////////////////////////
        let a1 = [0xE1, 0x01];
        let a2 = [0xEE, 0x00];

        let mut writer = BitStreamWriter::create(TEST_FILE_NAME).unwrap();
        writer.write_bit_sequence(&a1, 0).unwrap();
        writer.write_bit_sequence(&a2, 0).unwrap();

        let mut reader = BitStreamReader::open(TEST_FILE_NAME).unwrap();
        assert_eq!(reader.read_bit_sequence(64).unwrap(), []);

        //////////////////////////////////////////////
        // Read bytes
        //////////////////////////////////////////////
        let a1 = [0xE1, 0x01];
        let a2 = [0xEE, 0x77];

        let mut writer = BitStreamWriter::create(TEST_FILE_NAME).unwrap();
        writer.write_bit_sequence(&a1, 8).unwrap();
        writer.write_bit_sequence(&a2, 16).unwrap();

        let mut reader = BitStreamReader::open(TEST_FILE_NAME).unwrap();
        assert_eq!(reader.read_bit_sequence(16).unwrap(), [0xE1, 0xEE],);
        assert_eq!(reader.read_bit_sequence(8).unwrap(), [0x77],);
    }
}
