use crate::algorithms::{LzvDecoder, LzvEncoder, LzvHeader};
use crate::error::LzvError;
use bit_stream::{BitStreamReader, BitStreamWriter};
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use text_io::read;

pub const DEFAULT_ENCODED_FILE_EXTENSION: &str = "lzv_enc";
pub const DEFAULT_DECODED_FILE_EXTENSION: &str = "lzv_dec";

pub fn encode_file(input_file: &str, output_file: &str) -> Result<(), LzvError> {
    let buffer_size = 2048;

    let mut reader = BitStreamReader::open(input_file)?;
    let mut encoder = LzvEncoder::new(output_file)?;

    let mut buffer = reader.read_bit_sequence(8 * buffer_size)?;
    let i = 0;
    while !buffer.is_empty() {
        encoder.process(&buffer)?;

        buffer = reader.read_bit_sequence(8 * buffer_size)?;
    }

    encoder.finish()?;

    Ok(())
}

pub fn decode_file(input_file: &str, output_file: &str) -> Result<(), LzvError> {
    let mut decoder = LzvDecoder::new(input_file, output_file)?;

    decoder.process_all()?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum Options {
    Encode,
    Decode,
}

impl Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Encode => write!(f, "encode"),
            Options::Decode => write!(f, "decode"),
        }
    }
}

impl TryFrom<&str> for Options {
    type Error = LzvError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "encode" => Ok(Self::Encode),
            "decode" => Ok(Self::Decode),
            _ => Err(LzvError::InvalidInput),
        }
    }
}

pub fn run_encode_branch() {
    let mut input_file = String::new();
    while input_file.is_empty() {
        print!("Name a file you want to encode: ");
        input_file = read!("{}\n");

        if input_file.is_empty() {
            print!("You must provide a name. ");
        }
    }
    print!(
        "Name a file, where to store the result. You may leave it blank, to store data in {input_file}.{DEFAULT_ENCODED_FILE_EXTENSION}: "
    );
    let output_file: String = read!("{}\n");

    let output_file = match output_file.is_empty() {
        true => format!("{input_file}.{DEFAULT_ENCODED_FILE_EXTENSION}"),
        false => output_file,
    };

    if let Err(e) = encode_file(&input_file, &output_file) {
        println!("Error occurred: {}", e);
    }
}

pub fn run_decode_branch() {
    let mut input_file = String::new();
    while input_file.is_empty() {
        print!("Name a file you want to decode: ");
        input_file = read!("{}\n");

        if input_file.is_empty() {
            print!("You must provide a name. ");
        }
    }
    print!(
        "Name a file, where you want to store the result.\
        You may leave it blank, to store data\
        in {input_file}.{DEFAULT_DECODED_FILE_EXTENSION}: "
    );
    let output_file: String = read!("{}\n");

    let output_file = match output_file.is_empty() {
        true => format!("{input_file}.{DEFAULT_DECODED_FILE_EXTENSION}"),
        false => output_file,
    };

    if let Err(e) = decode_file(&input_file, &output_file) {
        println!("Error occurred: {}", e);
    }
}

pub fn run() {
    loop {
        println!("Options:");
        println!("  > {}", Options::Encode);
        println!("  > {}", Options::Decode);
        print!("Please chose what to do: ");
        let line: String = read!("{}\n");

        match Options::try_from(line.as_str()) {
            Ok(Options::Encode) => run_encode_branch(),
            Ok(Options::Decode) => run_decode_branch(),
            Err(_) => print!("Invalid option. Try again."),
        }

        println!("\n\n\n");
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{
        DEFAULT_DECODED_FILE_EXTENSION, DEFAULT_ENCODED_FILE_EXTENSION, decode_file, encode_file,
    };
    use chrono::Local;
    use std::fmt;
    use std::time::Instant;
    use tracing::info;
    use tracing_subscriber::fmt::{format::Writer, time::FormatTime};

    struct LocalTimer;

    impl FormatTime for LocalTimer {
        fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
            let now = Local::now();
            write!(w, "[{}]", now.format("%Y-%m-%d %H:%M:%S"))
        }
    }

    #[test]
    fn test_cli_algorithm() {
        _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_timer(LocalTimer)
            .with_target(true)
            .try_init();

        let file_name = "test_files/table3.xlsx";
        let input = &*format!("../{file_name}");
        let enc_output = &*format!("../{file_name}.{DEFAULT_ENCODED_FILE_EXTENSION}");
        let dec_output = &*format!(
            "../{file_name}.{DEFAULT_ENCODED_FILE_EXTENSION}.{DEFAULT_DECODED_FILE_EXTENSION}.xlsx"
        );

        info!("==============================");
        let start = Instant::now();
        encode_file(input, enc_output).unwrap();
        info!("Encoded in {:?}", start.elapsed());
        info!("==============================");
        let start = Instant::now();
        decode_file(enc_output, dec_output).unwrap();
        info!("Decoded in {:?}", start.elapsed());
    }
}
