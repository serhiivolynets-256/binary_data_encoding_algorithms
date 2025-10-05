use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};

#[derive(Debug)]
pub struct BitStreamWriter {
    file: File,
    bit_len: usize,
    current_byte: u8,
    bit_count: usize,
}

impl BitStreamWriter {
    pub fn create(path: &str) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            file,
            bit_len: 0,
            current_byte: 0u8,
            bit_count: 0,
        })
    }

    pub fn write_bit_sequence(&mut self, data: &[u8], bits_amount: usize) -> std::io::Result<()> {
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

        self.file.seek(SeekFrom::Start(self.bit_len as u64))?;
        self.file.write_all(&buffer)?;
        self.bit_len += to_take / 8;
        Ok(())
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
        let mut file_buffer = Vec::new();

        self.file.seek(SeekFrom::Start(0))?;
        self.file.read_to_end(&mut file_buffer)?;
        let available = self.bit_len.saturating_sub(self.read_pos);

        let to_read = bits_amount.min(available);
        let mut out = vec![0u8; (to_read + 7) / 8];
        for i in 0..to_read {
            let absolute_bit = self.read_pos + i;
            let absolute_byte = absolute_bit / 8;
            let absolute_bit_in_byte = absolute_bit % 8;

            let bit = (file_buffer[absolute_byte] >> absolute_bit_in_byte) & 1;

            if bit != 0 {
                out[i / 8] |= 1u8 << i % 8;
            }
        }

        self.read_pos += to_read;

        Ok(out)
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
        assert_eq!(
            reader.read_bit_sequence(11).unwrap(),
            [0xE1, 0x05],
        );
        assert_eq!(
            reader.read_bit_sequence(7).unwrap(),
            [0x3B],
        );

        //////////////////////////////////////////////
        // empty write
        //////////////////////////////////////////////
        let a1 = [0xE1, 0x01];
        let a2 = [0xEE, 0x00];

        let mut writer = BitStreamWriter::create(TEST_FILE_NAME).unwrap();
        writer.write_bit_sequence(&a1, 0).unwrap();
        writer.write_bit_sequence(&a2, 0).unwrap();

        let mut reader = BitStreamReader::open(TEST_FILE_NAME).unwrap();
        assert_eq!(
            reader.read_bit_sequence(64).unwrap(),
            []
        );

        //////////////////////////////////////////////
        // Read bytes
        //////////////////////////////////////////////
        let a1 = [0xE1, 0x01];
        let a2 = [0xEE, 0x77];

        let mut writer = BitStreamWriter::create(TEST_FILE_NAME).unwrap();
        writer.write_bit_sequence(&a1, 8).unwrap();
        writer.write_bit_sequence(&a2, 16).unwrap();

        let mut reader = BitStreamReader::open(TEST_FILE_NAME).unwrap();
        assert_eq!(
            reader.read_bit_sequence(16).unwrap(),
            [0xE1, 0xEE],
        );
        assert_eq!(
            reader.read_bit_sequence(8).unwrap(),
            [0x77],
        );
    }
}