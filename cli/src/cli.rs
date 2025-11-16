// use crate::{algorithms, error::Base64Error};
use crate::CliError;
use burroughs_wheeler_transformation::BurroughsWheelerEncode;
use huffman_codes;
use lzv_compression::algorithms::{LzvDecoder, LzvEncoder};
use lzv_compression::cli::{decode_file, encode_file};
use move_to_front::{MoveToFrontDecoder, MoveToFrontEncoder};
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use text_io::read;
use tracing::{debug, info};
use utils::init_tracing;

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug, Clone)]
struct Config {
    action: EncodeOption,
    input_file: String,
    output_file: String,
    algorithm: Algorithm,
    extensions: Vec<Extension>,
}

#[derive(Debug, Clone)]
enum EncodeOption {
    Encode,
    Decode,
}

#[derive(Debug, Clone)]
enum Algorithm {
    LZV,
    Huffman,
}

#[derive(Debug, Clone)]
enum Extension {
    MTF,
    BWT,
}

fn parse_input(s: String) -> Result<Config, CliError> {
    let tokens = s.split_whitespace().collect::<Vec<_>>();

    let action = match tokens[0] {
        "encode" => EncodeOption::Encode,
        "decode" => EncodeOption::Decode,
        _ => return Err(CliError::UnknownCommand(tokens[0].to_string())),
    };

    let input_file = tokens[1].to_string();
    let output_file = match tokens[2] {
        "default" => {
            let ext = match action {
                EncodeOption::Encode => "enc",
                EncodeOption::Decode => "dec",
            };

            format!("{}.{}", tokens[1], ext)
        }
        _ => tokens[2].to_string(),
    };

    let algorithm = match tokens[3] {
        "lzv" => Algorithm::LZV,
        "huffman" => Algorithm::Huffman,
        _ => return Err(CliError::UnknownAlgorithm(tokens[3].to_string())),
    };

    let mut extensions = vec![];
    if let Some(remaining_tokens) = tokens.get(4..) {
        for token in remaining_tokens {
            let extension = match *token {
                "mtf" => Extension::MTF,
                "bwt" => Extension::BWT,
                _ => return Err(CliError::UnknownExtension(token.to_string())),
            };
            extensions.push(extension);
        }
    }

    Ok(Config {
        action,
        input_file,
        output_file,
        algorithm,
        extensions,
    })
}

pub(crate) fn execute_command(config: &Config) -> Result<(), CliError> {
    debug!("Parsed config: {:?}", config);
    match config.action {
        EncodeOption::Encode => execute_encoding(config)?,
        EncodeOption::Decode => execute_decoding(config)?,
    }

    Ok(())
}

fn apply_mtf_extension_encode(config: &Config) -> io::Result<()> {
    debug!(
        "MTF encode: input_file: \"{}\", output_file: \"{}\"",
        config.input_file, config.output_file
    );
    let mut encoder = MoveToFrontEncoder::default();
    let mut infile = File::open(&config.input_file)?;
    let mut outfile = BufWriter::new(File::create(&config.output_file)?);

    let mut buffer = vec![0u8; CHUNK_SIZE];

    loop {
        let n = infile.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        let code = encoder.encode(&buffer[..n]);

        outfile.write_all(&code)?;
    }

    Ok(())
}

fn apply_mtf_extension_decode(config: &Config) -> io::Result<()> {
    debug!(
        "MTF decode: input_file: \"{}\", output_file: \"{}\"",
        config.input_file, config.output_file
    );
    let mut decoder = MoveToFrontDecoder::default();
    let mut infile = File::open(&config.input_file)?;
    let mut outfile = BufWriter::new(File::create(&config.output_file)?);

    let mut buffer = vec![0u8; CHUNK_SIZE];

    loop {
        let n = infile.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        let code = decoder.decode(&buffer[..n]);

        outfile.write_all(&code)?;
    }

    Ok(())
}

fn apply_bft_extension_encode(config: &Config) -> io::Result<()> {
    debug!(
        "BFT encode: input_file: \"{}\", output_file: \"{}\"",
        config.input_file, config.output_file
    );
    let mut infile = File::open(&config.input_file)?;
    let mut outfile = BufWriter::new(File::create(&config.output_file)?);

    let mut buffer = [0u8; CHUNK_SIZE];

    loop {
        let n = infile.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        if n != CHUNK_SIZE {
            outfile.write(&buffer[..n])?;
            return Ok(());
        }

        let (code_word, size) = BurroughsWheelerEncode::encode(&buffer);

        outfile.write_all(&code_word)?;
        outfile.write(&[size as u8])?;
    }

    Ok(())
}

fn apply_bft_extension_decode(config: &Config) -> io::Result<()> {
    debug!(
        "BFT decode: input_file: \"{}\", output_file: \"{}\"",
        config.input_file, config.output_file
    );
    let mut infile = File::open(&config.input_file)?;
    let mut outfile = BufWriter::new(File::create(&config.output_file)?);

    let mut buffer = [0u8; CHUNK_SIZE];
    let mut leftover = [0u8; 1];

    loop {
        let n1 = infile.read(&mut buffer)?;
        let n2 = infile.read(&mut leftover)?;
        if n1 + n2 == 0 {
            break;
        }

        if n1 + n2 != CHUNK_SIZE + 1 {
            outfile.write(&buffer[..n1])?;
            return Ok(());
        }

        let word = BurroughsWheelerEncode::decode(&buffer, leftover[0] as usize);

        outfile.write_all(&word)?;
    }

    Ok(())
}

fn apply_extensions(tmp_config: &mut Config) -> Result<(), CliError> {
    let output_file = tmp_config.output_file.clone();

    for (i, extension) in tmp_config.extensions.iter().enumerate() {
        tmp_config.output_file = format!("{}.tmp-{}", output_file, i);
        match extension {
            Extension::MTF => apply_mtf_extension_encode(&tmp_config)?,
            Extension::BWT => apply_bft_extension_encode(&tmp_config)?,
        }
        tmp_config.input_file = tmp_config.output_file.clone();
    }

    tmp_config.output_file = output_file;

    Ok(())
}

fn remove_extensions(tmp_config: &mut Config) -> Result<(), CliError> {
    let input_file = tmp_config.input_file.clone();
    let output_file = tmp_config.output_file.clone();

    for (i, extension) in tmp_config.extensions.iter().rev().enumerate() {
        if i == tmp_config.extensions.len() - 1 {
            tmp_config.output_file = output_file.clone();
        } else {
            tmp_config.output_file = format!("{}.dec.tmp-{}", input_file, i + 1);
        };

        match extension {
            Extension::MTF => apply_mtf_extension_decode(&tmp_config)?,
            Extension::BWT => apply_bft_extension_decode(&tmp_config)?,
        }
        tmp_config.input_file = tmp_config.output_file.clone();
    }

    tmp_config.output_file = output_file;

    Ok(())
}

fn execute_encoding(config: &Config) -> Result<(), CliError> {
    let mut tmp_config = config.clone();
    apply_extensions(&mut tmp_config)?;

    debug!(
        "{:?}: input_file: \"{}\", output_file: \"{}\"",
        tmp_config.algorithm, tmp_config.input_file, tmp_config.output_file
    );
    match tmp_config.algorithm {
        Algorithm::LZV => encode_file(&tmp_config.input_file, &tmp_config.output_file)?,
        Algorithm::Huffman => {
            huffman_codes::algorithms::encode(&tmp_config.input_file, &tmp_config.output_file)?
        }
    }

    Ok(())
}

fn execute_decoding(config: &Config) -> Result<(), CliError> {
    let mut tmp_config = config.clone();

    if config.extensions.is_empty() {
        debug!(
            "{:?}: input_file: \"{}\", output_file: \"{}\"",
            config.algorithm, config.input_file, config.output_file
        );
        match config.algorithm {
            Algorithm::LZV => decode_file(&config.input_file, &config.output_file)?,
            Algorithm::Huffman => {
                huffman_codes::algorithms::decode(&config.input_file, &config.output_file)?
            }
        }

        return Ok(());
    }

    let output_file = format!("{}.tmp-{}", tmp_config.output_file, 0);

    debug!(
        "{:?}: input_file: \"{}\", output_file: \"{}\"",
        config.algorithm, config.input_file, output_file
    );
    match config.algorithm {
        Algorithm::LZV => decode_file(&config.input_file, &output_file)?,
        Algorithm::Huffman => huffman_codes::algorithms::decode(&config.input_file, &output_file)?,
    }

    tmp_config.input_file = output_file;
    tmp_config.output_file = config.output_file.clone();
    remove_extensions(&mut tmp_config)?;

    Ok(())
}

fn run_help() {
    println!(
        "Help:
    Interface: <encode | decode> <input_file> <output_file> <algorithm> [extensions...]

    Arguments:
        encode | decode - Mode of operation.
        input_file - Path to the file to read.
        output_file - Path to the file to write to or \"default\" for a default one.
        algorithm - One of: \
                lzv       - LZW encoding
                huffman   - Huffman encoding
        extensions - Zero or more of:
                mtf       - Move-to-Front transform
                bwt       - Burroughs-Wheeler transform

    Examples:
        encode test.txt test.enc lzv bwt mtf
        decode test.enc test.dec lzv bwt mtf

    "
    );
}

pub fn run() {
    init_tracing();
    run_help();

    loop {
        print!("Please chose what to do: ");
        let line: String = read!("{}\n");

        match parse_input(line) {
            Ok(config) => match execute_command(&config) {
                Ok(_) => {}
                Err(err) => {
                    println!("Failed to execute command: {}", err);
                }
            },
            Err(err) => {
                println!("Invalid input: {err}. Try again\n");
                continue;
            }
        };

        println!();
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{Algorithm, Config, EncodeOption, Extension, execute_command};
    use lzv_compression::cli::{decode_file, encode_file};
    use std::time::Instant;
    use tracing::{debug, info};
    use utils::init_tracing;

    fn run_encode_decode(file_name: &str) {
        let input = &*format!("../{file_name}");
        let enc_output = &*format!("../{file_name}.enc");
        let dec_output = &*format!("../{file_name}.dec.txt");

        let mut config = Config {
            action: EncodeOption::Encode,
            input_file: input.to_string(),
            output_file: enc_output.to_string(),
            algorithm: Algorithm::Huffman,
            extensions: vec![Extension::BWT, Extension::MTF],
        };

        info!("=============== {{ ENCODE: {} }} ===============", input);
        let start = Instant::now();
        execute_command(&config).unwrap();
        info!("Encoded in {:?}", start.elapsed());

        config.action = EncodeOption::Decode;
        config.input_file = enc_output.to_string();
        config.output_file = dec_output.to_string();

        info!(
            "=============== {{ Decode: {} }} ===============",
            enc_output
        );
        let start = Instant::now();
        execute_command(&config).unwrap();
        info!("Decoded in {:?}", start.elapsed());
    }

    #[test]
    fn test_cli() {
        init_tracing();

        // let file_name = "test_files/file1.txt";
        let file_name = "test.txt";

        run_encode_decode(file_name);
    }
}
